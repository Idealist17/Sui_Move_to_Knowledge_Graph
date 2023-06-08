use crate::move_ir::sbir_generator::*;
use crate::detect::detect::{detect_unchecked_return, 
    detect_unused_private_functions,
    detect_unused_constants,
};

#[test]
fn test_get_from_bytecode_modules() {
    let dir = "./testdata/examples_mv/aptos/";
    let bc = Blockchain::Aptos;
    let ms = MoveScanner::new(dir, bc);

    // unchecked return
    for (_, func) in &ms.functions {
        if detect_unchecked_return(&func) {
            println!("{}", func.module_name.display(ms.env.symbol_pool()));
            println!("{}", func.name.display(ms.env.symbol_pool()));
        }
    }

    // unused private function
    let unused_private_functions = detect_unused_private_functions(&ms);
    for fun in unused_private_functions {
        let fname = ms.env.get_function(*fun).get_full_name_str();
        println!("Unused Private function: {}", fname);
    }

    // unused constant
    detect_unused_constants(&ms);
}
