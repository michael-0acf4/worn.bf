use std::path::PathBuf;

use clap::Parser;

/// worn (write once run nowhere)
/// the "ultimate" Brainfuck emitter/compiler/optimizer
#[derive(Parser, Debug)]
#[command(name = "worn", author = "Michael", about)]
struct CompilerArgs {
    /// Input source file
    #[arg()]
    file: PathBuf,
    /// Sets the output file
    #[arg(short)]
    output: Option<String>,
    /// Compile worn lang to Brainf*ck
    #[arg(short, long)]
    worn: bool,
    /// Enables optimization
    #[arg(short = 'x', long)]
    optimize: bool,
}

fn main() {
    let args = CompilerArgs::parse();
    println!("{:#?}", args);
}
