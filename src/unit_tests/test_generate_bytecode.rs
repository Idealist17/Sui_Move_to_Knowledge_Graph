use std::path::PathBuf;
use std::str::FromStr;

use crate::move_ir::generate_bytecode::*;
use crate::utils::utils::compile_module;
use move_stackless_bytecode::stackless_bytecode::Bytecode;

#[test]
fn test_generate_bytecode() {
    let filename = PathBuf::from_str("/Users/lteng/Movebit/detect/build/movebit/bytecode_modules/witness_user.mv").unwrap();
    let cm = compile_module(filename);
    for fd in &cm.function_defs { 
        let mut bg = StacklessBytecodeGenerator::new(&cm, fd);
        bg.generate_function();
        let bytecodes = bg.code.clone();
        println!("{}", bytecodes.len());
        // let bytecodes = bg.code;
        // let label_offsets = Bytecode::label_offsets(&bytecodes);
        // for (offset, code) in bg.code.iter().enumerate() {
        //     println!(
        //         "{}",
        //         format!("{:>3}: {}", offset, code.display(&target, &label_offsets))
        //     );
        // }
    }
}