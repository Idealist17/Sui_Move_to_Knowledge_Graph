use crate::utils::utils::is_dep_module;
use crate::move_ir::sbir_generator::{
    Function,
    MoveScanner,
};
use move_binary_format::internals::ModuleIndex;
use move_stackless_bytecode::stackless_bytecode::{
    Bytecode, Operation
};
use move_model::{model::{FunId, QualifiedId}};
use move_binary_format::file_format::Visibility;
use petgraph::Direction;
use move_binary_format::file_format::{Bytecode as MoveBytecode};


pub fn detect_unchecked_return(func: &Function) -> bool {
    let mut res = vec![];
    // let cnt = func.bytecodes.len();
    for (code_offset, bytecode) in func.bytecodes.iter().enumerate() {
        match &bytecode {
            Bytecode::Call(_, dsts , Operation::Function(_, _, _), _, _) => {
                let ret_cnt = dsts.len();
                let mut flag = if ret_cnt == 0 {false} else {true};
                for (id , dst) in dsts.iter().enumerate() {
                    // 从后向前依次 pop(destory) 掉函数返回值
                    match &func.bytecodes[code_offset+ret_cnt-id] {
                        Bytecode::Call(_, _, Operation::Destroy, destory_srcs, _) => {
                            if destory_srcs[0] == *dst {
                                continue;
                            } else {
                                flag = false;
                                break;
                            }
                        },
                        _ => {
                            flag = false;
                            break;
                        }
                    }
                }
                res.push(flag);
            },
            _ => {
                continue;
            }
        }
    }
    res.contains(&true)
}


fn get_unused_functions(ms: &MoveScanner) -> Vec<&QualifiedId<FunId>> {
    let mut unused_funs: Vec<&QualifiedId<FunId>> = vec![];
    for (fid, nid) in ms.fun_map.iter() {
        if is_dep_module(&(ms.env).get_module(fid.module_id)) {
            continue;
        }
        // 调用边，即入边
        let neighbors = ms.call_graph.neighbors_directed(*nid, Direction::Incoming);
        if neighbors.into_iter().next().is_none() {
            unused_funs.push(fid);
        }
    }
    unused_funs
}

pub fn detect_unused_private_functions(ms: &MoveScanner) -> Vec<&QualifiedId<FunId>> {
    let mut unused_private_functions: Vec<&QualifiedId<FunId>> = vec![];
    let unused_funs = get_unused_functions(ms);
    for fun in unused_funs {
        let fun_env = ms.env.get_function(*fun);
        let visibility = fun_env.visibility();
        match visibility {
            Visibility::Private => unused_private_functions.push(fun),
            _ => {},

        }
    }
    unused_private_functions
}

pub fn detect_unused_constants(ms: &MoveScanner) {
    for module in ms.env.module_data.iter() {
        if is_dep_module(&(ms.env).get_module(module.id)) {
            continue;
        }
        let cm = &module.module;
        let const_pool = &cm.constant_pool;
        let len = const_pool.len();
        let mut used = vec![false; len];
        for fun in cm.function_defs.iter() {
            if let Some(codes) = &fun.code {
                for code in codes.code.iter() {
                    match code {
                        MoveBytecode::LdConst(idx) => {
                            used[idx.into_index()] = true;
                        },
                        _ => {},
                    }
                };
            } else {
                continue;
            }
        }
        println!("{:?}", used);
    }
}