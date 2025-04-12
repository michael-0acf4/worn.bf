use clap::Parser;
use parser::{ast::Instruction, ast::Reconstruct, parse_program};
use std::{path::PathBuf, process::exit};

mod parser;
mod sbf;

/// WORN (Write Once, Run Nowhere)
/// the "ultimate" Brainfuck emitter/compiler/optimizer
#[derive(Parser, Debug)]
#[command(name = "worn", author = "Michael", about)]
struct CompilerArgs {
    /// Input source file
    #[arg()]
    file: PathBuf,
    /// Set the output file
    #[arg(short)]
    output: Option<String>,
    /// Compile worn lang to Brainf*ck
    #[arg(short, long)]
    worn: bool,
    /// Enable optimization
    #[arg(short = 'x', long)]
    optimize: bool,
}

fn main() {
    let args = CompilerArgs::parse();

    let content = std::fs::read_to_string(args.file).expect("Unable to read file");
    match parse_program(&content) {
        Ok(code) => {
            println!("{}", code.reconstruct());
        }
        Err(e) => {
            panic!("{e:?}");
        }
    }
}
