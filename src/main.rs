use clap::Parser;
use parser::{ast::Reconstruct, parse_program};
use sbf::SBFEmitter;
use std::path::PathBuf;

mod optimizer;
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
    /// Enable optimization
    #[arg(short = 'x', long)]
    optimize: bool,
}

fn main() -> Result<(), String> {
    let args = CompilerArgs::parse();

    let content = std::fs::read_to_string(args.file).expect("Unable to read file");
    parse_program(&content).and_then(|program| {
        println!("{}", program.reconstruct());
        let mut emitter = SBFEmitter::new(program);
        emitter.compile().map_err(|e| e.to_string())?;
        println!(
            "\n------------\n\n{}",
            emitter
                .finalize()
                .map(|bi| bi.reconstruct())
                .map_err(|e| e.to_string())?
        );
        Ok(())
    })
}
