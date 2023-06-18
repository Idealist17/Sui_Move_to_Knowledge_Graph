use std::path::PathBuf;
use std::str::FromStr;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::FunctionDefinitionIndex;
use move_binary_format::views::FunctionDefinitionView;

use crate::detect::detect2::detect_overflow;
use crate::detect::detect3::detect_precision_loss;
use crate::detect::detect7::detect_unnecessary_type_conversion;
use crate::detect::detect8::detect_unnecessary_bool_judgment;
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

#[test]
fn test_detect_precision_loss() {
    let filename = PathBuf::from_str("/home/yule/Movebit/detect/build/movebit/bytecode_modules/precision.mv").unwrap();
    let cm = compile_module(filename);
    let mut stbgr = StacklessBytecodeGenerator::new(&cm);
    stbgr.generate_function();
    for (idx, function) in stbgr.functions.iter().enumerate() {
        // println!("{:?}",function.code);
        if detect_precision_loss(function, &stbgr.symbol_pool) {
            let name = cm.identifier_at(cm.function_handle_at(cm.function_defs[idx].function).name);
            println!("{} : {}", name, "precision loss");
        }
    }
}

#[test]
fn test_detect_unnecessary_type_conversion() {
    let filename = PathBuf::from_str("/home/yule/Movebit/detect/build/movebit/bytecode_modules/unnecessary_type_conversion.mv").unwrap();
    let cm = compile_module(filename);
    let mut stbgr = StacklessBytecodeGenerator::new(&cm);
    stbgr.generate_function();
    for (idx, function) in stbgr.functions.iter().enumerate() {
        // println!("{:?}", function.code);
        // println!("{:?}", &function.local_types);
        if detect_unnecessary_type_conversion(function, &function.local_types) {
            let name = cm.identifier_at(cm.function_handle_at(cm.function_defs[idx].function).name);
            println!("{} : {}", name, "unnecessary type conversion");
        }
    }
}

#[test]
fn test_detect_unnecessary_bool_judgment() {
    let filename = PathBuf::from_str("/home/yule/Movebit/detect/build/movebit/bytecode_modules/unnecessary_bool_judgment.mv").unwrap();
    let cm = compile_module(filename);
    let mut stbgr = StacklessBytecodeGenerator::new(&cm);
    stbgr.generate_function();
    for (idx, function) in stbgr.functions.iter().enumerate() {
        // println!("{:?}", function.code);
        if detect_unnecessary_bool_judgment(function, &function.local_types) {
            let name = cm.identifier_at(cm.function_handle_at(cm.function_defs[idx].function).name);
            println!("{} : {}", name, "unnecessary bool judgment");
        }
    }
}

#[test]
fn test_detect_overflow() {
    let filename = PathBuf::from_str("/home/yule/Movebit/detect/build/movebit/bytecode_modules/overflow.mv").unwrap();
    let cm = compile_module(filename);
    let mut stbgr = StacklessBytecodeGenerator::new(&cm);
    stbgr.generate_function();
    for (idx, function) in stbgr.functions.iter().enumerate() {
        println!("{:?}", function.code);
        let func_define = cm.function_def_at(FunctionDefinitionIndex::new(idx as u16));
        let view = FunctionDefinitionView::new(&cm, &func_define);
        let parameters_len = view.parameters().len();
        if detect_overflow(function, &function.local_types) {
            let name = cm.identifier_at(cm.function_handle_at(cm.function_defs[idx].function).name);
            println!("{} : {}", name, "overflow");
        }
    }
}