// RepeatedFunctionCall
use crate::{
    move_ir::{generate_bytecode::FunctionInfo, packages::Packages, utils},
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::{model::FunId, symbol::SymbolPool};
use move_stackless_bytecode::{
    stackless_bytecode::{Bytecode, Operation, AssignKind},
    stackless_control_flow_graph::BlockContent,
};

pub struct Detector10<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}
use std::collections::{HashMap, HashSet};
impl<'a> AbstractDetector<'a> for Detector10<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Minor, DetectKind::RepeatedFunctionCall),
        }
    }
    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            let symbol_pool = &stbgr.symbol_pool;
            for (idx, function) in stbgr.functions.iter().enumerate() {
                // 跳过 native 函数
                if utils::is_native(idx, stbgr) {
                    continue;
                }
                // 通过 funid 得到函数名，并构造最终输入：f1(f2,f3)，指函数 f1 中重复调用了 f2 和 f3
                if let Some(funids) = self.detect_repeated_function_call(function) {
                    let res = format!(
                        "{}({})",
                        function.name,
                        funids
                            .into_iter()
                            .map(|funid| { self.get_function_name(symbol_pool, &funid) })
                            .collect::<Vec<String>>()
                            .join(",")
                    );
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector10<'a> {
    fn detect_repeated_function_call(&self, function: &FunctionInfo) -> Option<Vec<FunId>> {
        let cfg = function.cfg.as_ref().unwrap();
        let code = &function.code;
        // stack 用于迭代 dfs，其栈顶元素为下一个要访问的区块id（block_id）
        let mut stack = Vec::new();
        // path 作为stack 中压入的区块的历史记录
        let mut path = Vec::new();
        // func_map 中，key 为 block_id，value 为该代码块中调用的函数id（FunId）
        let mut func_map = HashMap::new();
        // 代码入口块
        let entry_block_id = cfg.entry_block();
        stack.push(&entry_block_id);
        path.push(&entry_block_id);
        let mut repeated_funids = Vec::new();
        // dfs 深度遍历
        while !stack.is_empty() {
            // 当前块以访问，从 stack 中弹出，注意，path 并不会弹出
            let block_id = stack.pop().unwrap();
            let block_content = cfg.content(*block_id);
            cfg.successors(*block_id)
                .iter()
                .for_each(|successor_block_id| {

                    // 若 path 中不存在后继块，则将后继块压入 stack 和 path 中
                    // 这是为了避免 while 循环：例如 path=[0,1]，此时 1 的后继块为 0，那就说明有循环，应该跳过循环
                    if !path.contains(&successor_block_id){
                        stack.push(successor_block_id);
                        path.push(successor_block_id);
                    }
                });
            match *block_content {
                // 当前区块为代码块
                BlockContent::Basic { lower, upper } => {
                    let mut funids = Vec::new();
                    // 获取当前代码块调用的所有函数id，将其写入 func_map 中
                    for (offset, bytecode) in
                        (lower..=upper).zip(&code[(lower as usize)..=(upper as usize)])
                    {
                        match bytecode {
                            Bytecode::Call(attr_id, dsts, op, srcs, _) => match op {
                                Operation::Function(mid, funid, tys) => {
                                    funids.push(funid);
                                }
                                _ => {}
                            },
                            // copy move sstore 均为赋值语句
                            // Bytecode::Assign(_, dst,src ,askind ) =>{

                            // },
                            _ => {}
                        }
                    }
                    func_map.insert(block_id, funids);
                    // println!("{:?}", block_content);
                }

                // 当前代码块为入口块或结束块。（其实写的是结束块的处理逻辑）
                BlockContent::Dummy => {
                    // 判断 func_map 中是否存在重复的函数id
                    // seen_funids 中存储已经出现过一次的 funid
                    let mut seen_funids = HashSet::new();

                    // 拿出所有的 FunId
                    for funids in func_map.values() {
                        for &funid in funids {
                            if seen_funids.insert(funid) || repeated_funids.contains(funid) {
                                continue
                            }
                            repeated_funids.push(*funid);
                        }
                    }

                    /*
                     *  设 cfg 为 0->1 1->2 1-3 2->4 3->4
                     *  该 cfg 存在两条路径 0->1->2->4 和 0->1->3->4
                     *  执行完 0->1->2->4 后：
                     *  stack=[3]
                     *  path =[0,1,3,2,4]
                     *  func_map={0:[...],1:[...],2:[...],4:[...]}
                     *  因此如果我们想分析执行分支 0->1->3->4 是否存在重复的函数调用，需要删除func_map中 2、4 的记录，保留 0、1 的记录
                     *  那么，先从 path 中弹出 4，再弹出 2，最后发现其栈顶和stack的栈顶元素相同，则不再进行删除操作。
                     *  如此，成功在 func_map 中删除了 2、4，而保留了0、1的记录
                     *  TIP: 在实际执行中，0 和 4 均为 dummpy 的空块
                     * */
                    if let Some(&stack_block_id) = stack.last() {
                        while let Some(&path_block_id) = path.last() {
                            if stack_block_id == path_block_id {
                                break;
                            }
                            path.pop();
                            func_map.remove(&path_block_id);
                        }
                    }
                }
            }
        }
        if repeated_funids.is_empty() {
            None
        } else {
            Some(repeated_funids)
        }
    }

    fn get_function_name(&self, symbol_pool: &SymbolPool, funid: &FunId) -> String {
        funid.symbol().display(symbol_pool).to_string()
    }

    // fn bfs(&self, function: &FunctionInfo) {
    //     let mut queue = VecDeque::new();
    //     let cfg = function.cfg.as_ref().unwrap();
    //     let entry_block_id = cfg.entry_block();
    //     queue.push_front(entry_block_id);
    //     let mut visited = Vec::new();
    //     while !queue.is_empty() {
    //         let block_id = queue.pop_back().unwrap();
    //         let block_content = cfg.content(block_id);
    //         cfg.successors(block_id).iter().for_each(
    //             |successor_block_id| {
    //                 // 如果后继节点未被访问过
    //                 if !visited.contains(successor_block_id){
    //                     visited.push(successor_block_id.clone());
    //                     queue.push_front(successor_block_id.clone());
    //                 }
    //             },
    //         );
    //         match block_content {
    //             BlockContent::Basic { lower, upper } => {
    //                 println!("{:?}", block_content);
    //             }
    //             BlockContent::Dummy => {
    //                 println!("{:?}", block_content);
    //             }
    //         }
    //     }
    // }
}
