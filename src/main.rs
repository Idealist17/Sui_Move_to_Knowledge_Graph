#![allow(non_snake_case)]
use MoveScanner::{
    cli::parser::{Cli, SubCommands},
    scanner::{detectors::Detectors, printer::Printer},
    utils::utils
};

use clap::Parser;

fn main() {
    utils::print_logo();
    let cli = Cli::parse();
    match &cli.command {
        Some(SubCommands::Printer) => {
            // todo: 代码优化
            let mut printer = Printer::new(cli.args);
            printer.run();
        }
        // 默认 Detector
        _ => {
            let mut detector = Detectors::new(cli.args);
            detector.run();
            detector.output_result();
        }
    }
}
