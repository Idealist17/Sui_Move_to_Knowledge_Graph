use clap::Parser;
use MoveScanner::cli::parser::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        None => {
            println!(
                "no app was used for dealing with {}",
                cli.filedir
            )
        }
        Some(Commands::Printer { printer }) => {
            println!(
                "myapp printer was used for dealing with {}, name is: {:?}",
                cli.filedir, printer
            )
        }
        Some(Commands::Detection { detection }) => {
            println!(
                "myapp detection was used for dealing with {}, name is: {:?}",
                cli.filedir, detection
            )
        }
    }
}
