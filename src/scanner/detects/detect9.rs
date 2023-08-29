// recursive_function_call

use crate::{
    move_ir::{generate_bytecode::StacklessBytecodeGenerator, packages::Packages, utils},
    scanner::{detectors::AbstractDetector, result::*},
};
use move_stackless_bytecode::stackless_bytecode::{Bytecode, Operation};
use std::rc::Rc;
pub struct Detector9<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector9<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Major, DetectKind::RecursiveFunctionCall),
        }
    }
    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            for (idx, _function) in stbgr.functions.iter().enumerate() {
                // 跳过 native 函数
                if utils::is_native(idx, stbgr) {
                    continue;
                }
                if let Some(res) = self.detect_recursive_function_call(stbgr, idx) {
                    self.content.result.get_mut(mname).unwrap().push(res);
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector9<'a> {
    pub fn detect_recursive_function_call(
        &self,
        stbgr: &StacklessBytecodeGenerator,
        idx: usize,
    ) -> Option<String> {
        let function = &stbgr.functions[idx];
        let mut ret_flag = false;
        let symbol_pool = &stbgr.symbol_pool;
        for (_, bytecode) in function.code.iter().enumerate() {
            match &bytecode {
                Bytecode::Call(_, _, Operation::Function(mid, fid, _), _, _) => {
                    if symbol_pool.string(fid.symbol()) == Rc::from(function.name.clone()) &&  mid.to_usize() == 0 {
                        ret_flag = true;
                    } else {
                        // 函数调用图
                        let call_g = &stbgr.call_graph;
                    }
                }
                _ => {}
            }
        }
        if ret_flag {
            let curr_func_name = utils::get_function_name(idx, stbgr);
            Some(curr_func_name)
        } else {
            None
        }
    }
}
