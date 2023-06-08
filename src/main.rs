use std::path::PathBuf;

use clap::Parser;
use MoveScanner::{cli::parser::*, 
    move_ir::sbir_generator::{MoveScanner as Mc, Blockchain},
};

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Printer { printer }) => {
            let dir = "./testdata/examples_mv/aptos/";
            let bc = Blockchain::Aptos;
            let ms = Mc::new(dir, bc);
            match printer {
                Some(Infos::CFG) => {
                    let (qid, _) = ms.functions.first_key_value().unwrap();
                    let file_path = Some(PathBuf::from("cfg.dot"));
                    let _ = ms.get_cfg(qid, file_path);
                },
                Some(Infos::CompileModule) => {
                    let (qid, _) = ms.functions.first_key_value().unwrap();
                }, 
                Some(Infos::IR) => {
                    let mut text = String::new();
                    text += &ms.print_targets_for_test();
                    println!("{}", text);
                },
                _ => {

                },
            }
        },
        Some(Commands::Detection { detection }) => {
            println!(
                "myapp detection was used for dealing with {}, name is: {:?}",
                cli.filedir, detection
            )
        },
        None => {
            println!(
                "no app was used for dealing with {}",
                cli.filedir
            )
        }
    }
}
