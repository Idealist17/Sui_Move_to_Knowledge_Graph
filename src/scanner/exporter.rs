use crate::move_ir::packages::Packages;
use crate::move_ir::generate_bytecode::StacklessBytecodeGenerator;
use crate::scanner::graph::{GraphOutput, NodeWrapper, ModuleNode, FunctionNode, StructNode, EdgeWrapper};
use move_binary_format::access::ModuleAccess;
use move_model::model::{FunId, ModuleId, QualifiedId};


pub struct GraphExporter;

use crate::scanner::result::Result;
use regex::Regex;

impl GraphExporter {
    pub fn export(packages: &Packages, result: &Result) -> GraphOutput {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for (module_name_str, stbgr) in packages.get_all_stbgr() {
            // Load source code if available
            let mut source_content_opt = None;
            if let Some(mod_info) = result.modules.get(module_name_str) {
                 if let Some(loc_str) = &mod_info.location {
                     // loc_str is "path:line"
                     if let Some((path_str, _)) = loc_str.rsplit_once(':') {
                          if let Ok(content) = std::fs::read_to_string(path_str) {
                              source_content_opt = Some(content);
                          }
                     }
                 }
            }

            // Helper to stringify Loc
            let get_src = |loc: &move_model::model::Loc, name: &str, type_desc: &str| -> String {
                 if let Some(source_content) = &source_content_opt {
                     let start = loc.span().start().to_usize();
                     let end = loc.span().end().to_usize();
                     if start < end && end <= source_content.len() {
                         return source_content[start..end].to_string();
                     } else {
                         return extract_definition(source_content, type_desc, name);
                     }
                 }
                 String::new()
            };

            // 1. Module Node
            let mod_id_str = module_name_str.clone();
            let address = if let Some(idx) = mod_id_str.find("::") {
                mod_id_str[..idx].to_string()
            } else {
                "0x0".to_string()
            };
            let name = if let Some(idx) = mod_id_str.find("::") {
                mod_id_str[idx+2..].to_string()
            } else {
                mod_id_str.clone()
            };

            nodes.push(NodeWrapper::Module(ModuleNode {
                id: mod_id_str.clone(),
                address,
                name,
            }));

            // 2. Struct Nodes
            for (struct_id, struct_data) in &stbgr.module_data.struct_data {
                let s_name = stbgr.symbol_pool.string(struct_data.name).to_string();
                let full_struct_id = format!("{}::{}", mod_id_str, s_name);
                
                let def_idx = stbgr.module_data.struct_idx_to_id.iter().find(|(_, id)| **id == *struct_id).map(|(idx, _)| *idx);
                let mut abilities = Vec::new();
                let mut is_resource = false;
                if let Some(idx) = def_idx {
                     abilities = crate::move_ir::utils::get_struct_abilities_strs(stbgr.module, idx);
                     if abilities.contains(&"key".to_string()) {
                         is_resource = true;
                     }
                }
                
                let source = get_src(&struct_data.loc, &s_name, "struct");

                nodes.push(NodeWrapper::Struct(StructNode {
                    id: full_struct_id.clone(),
                    module_id: mod_id_str.clone(),
                    name: s_name,
                    abilities,
                    is_resource,
                    source,
                }));

                edges.push(EdgeWrapper::Defines {
                    from: mod_id_str.clone(),
                    to: full_struct_id,
                });
            }

            // 3. Function Nodes and Call Graph
            for function in &stbgr.functions {
                let f_name = function.name.clone();
                let full_func_id = format!("{}::{}", mod_id_str, f_name);
                
                let def = stbgr.module.function_def_at(move_binary_format::file_format::FunctionDefinitionIndex(function.idx as u16));
                let is_native = def.is_native();
                let visibility = match def.visibility {
                    move_binary_format::file_format::Visibility::Public => "public",
                    move_binary_format::file_format::Visibility::Friend => "friend",
                    move_binary_format::file_format::Visibility::Private => "private",
                }.to_string();
                
                // Extract Source
                let func_def_idx = move_binary_format::file_format::FunctionDefinitionIndex(function.idx as u16);
                let func_id = stbgr.module_data.function_idx_to_id[&func_def_idx];
                let func_data = &stbgr.module_data.function_data[&func_id];
                let source = get_src(&func_data.loc, &f_name, "fun");

                nodes.push(NodeWrapper::Function(FunctionNode {
                    id: full_func_id.clone(),
                    module_id: mod_id_str.clone(),
                    name: f_name.clone(),
                    visibility,
                    is_native,
                    arg_count: function.args_count,
                    source,
                }));

                edges.push(EdgeWrapper::Defines {
                    from: mod_id_str.clone(),
                    to: full_func_id.clone(),
                });
            }

            // 4. Extract Calls from Graph
            for edge in stbgr.call_graph.edge_indices() {
                let (source, target) = stbgr.call_graph.edge_endpoints(edge).unwrap();
                let source_qid = stbgr.call_graph.node_weight(source).unwrap();
                let target_qid = stbgr.call_graph.node_weight(target).unwrap();

                let source_str = resolve_qid(stbgr, source_qid);
                let target_str = resolve_qid(stbgr, target_qid);

                if source_qid.module_id.to_usize() == 0 { 
                     edges.push(EdgeWrapper::Calls {
                        from: source_str,
                        to: target_str,
                    });
                }
            }
        }

        GraphOutput { nodes, edges }
    }
}

fn resolve_qid(stbgr: &StacklessBytecodeGenerator, qid: &QualifiedId<FunId>) -> String {
    let module_name = &stbgr.module_names[qid.module_id.to_usize()];
    let m_str = module_name.display(&stbgr.symbol_pool).to_string();
    let f_str = stbgr.symbol_pool.string(qid.id.symbol()).to_string();
    format!("{}::{}", m_str, f_str)
}

fn extract_definition(source: &str, kind: &str, name: &str) -> String {
    let pattern = format!(r"\b{}\s+{}\b", kind, regex::escape(name));
    if let Ok(re) = Regex::new(&pattern) {
        if let Some(mat) = re.find(source) {
            let start_idx = mat.start();
            let chars: Vec<char> = source[start_idx..].chars().collect();
            let mut brace_count = 0;
            let mut found_brace = false;
            let mut end_offset = 0;
            
            for (i, c) in chars.iter().enumerate() {
                if *c == '{' {
                    brace_count += 1;
                    found_brace = true;
                } else if *c == '}' {
                    brace_count -= 1;
                    if brace_count == 0 && found_brace {
                        end_offset = i + 1;
                        break;
                    }
                } else if *c == ';' && !found_brace {
                     end_offset = i + 1;
                     break;
                }
            }
            // Fallback: if no brace found but loop ended (eof), take all
            if end_offset == 0 && !chars.is_empty() {
                 // Maybe scan until empty line or something? 
                 // For now, if we found match but no terminator, return remaining line?
                 // Safer to return nothing or line.
                 // Actually, if we hit EOF while braces open > 0, it's invalid code but we can return what we have.
                 // If brace_count == 0 and !found_brace and no ';', maybe it's `native fun X` without `;` (impossible).
            }

            if end_offset > 0 {
                 return source[start_idx .. start_idx + end_offset].to_string();
            }
        }
    }
    String::new()
}
