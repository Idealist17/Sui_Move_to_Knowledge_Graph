use std::path::PathBuf;
use std::str::FromStr;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::FunctionDefinitionIndex;

use crate::move_ir::{
    bytecode_display, 
    generate_bytecode::StacklessBytecodeGenerator
};
use crate::utils::utils::compile_module;
use crate::detect::detect1::detect_unchecked_return;


#[test]
fn test_detect_unchecked_return() {
    let filename = PathBuf::from_str("/home/yule/Movebit/detect/build/movebit/bytecode_modules/unchecked_return.mv").unwrap();
    let cm = compile_module(filename);
    let mut stbgr = StacklessBytecodeGenerator::new(&cm);
    stbgr.generate_function();
    for (idx, function) in stbgr.functions.iter().enumerate() {
        if detect_unchecked_return(function) {
            let name = cm.identifier_at(cm.function_handle_at(cm.function_defs[idx].function).name);
            println!("{} : {}", name, "unchecked return");
        }
    }
}
