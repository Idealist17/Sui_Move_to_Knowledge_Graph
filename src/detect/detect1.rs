// unckecked_return

use std::cmp;

use crate::move_ir::generate_bytecode::FunctionInfo;
use move_stackless_bytecode::stackless_bytecode::{
    Bytecode, Operation
};


pub fn detect_unchecked_return(function: &FunctionInfo) -> bool {
    let mut ret_flag = false;
    for (code_offset, bytecode) in function.code.iter().enumerate() {
        match &bytecode {
            Bytecode::Call(_, dsts , Operation::Function(_, _, _), _, _) => {
                let ret_cnt = dsts.len();
                // 函数没有返回值 false
                if ret_cnt == 0 {
                    continue;
                } else {
                    for pop in function.code[code_offset + 1..cmp::min(function.code.len(),code_offset + ret_cnt + 1)].iter() {
                        match pop {
                            Bytecode::Call(_, _, Operation::Destroy, _, _) => {
                                ret_flag = true;
                                break;
                            },
                            _ => {
                                continue;
                            }
                        }
                    }
                }
            },
            _ => {
                continue;
            }
        }
    }
    ret_flag
}
