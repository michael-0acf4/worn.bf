use clap::Parser;
use parser::{ast::Reconstruct, parse_program};
use sbf::SBFEmitter;
use std::path::PathBuf;

mod parser;
mod sbf;

#[cfg(test)]
mod tests;

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

            let mut emitter = SBFEmitter::new(&content).unwrap();
            emitter.compile().unwrap();
            println!("\n------------\n\n{}", emitter.finalize().unwrap())
        }
        Err(e) => {
            panic!("{e:?}");
        }
    }
}
