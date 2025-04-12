use clap::Parser;
use sbf::Program;
use std::{path::PathBuf, process::exit};

mod intr;
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
    let mut program = Program::from(&content);
    match program.parse() {
        Ok(code) => {
            for instr in code {
                println!("{}", instr.reconstruct());
            }
        }
        Err(e) => {
            match e {
                sbf::Error::UnexpectedToken { message, span }
                | sbf::Error::UnexpectedEof { message, span } => {
                    eprintln!("Pared\n {}\n", &content[..span.start]);
                    eprintln!("{:?}: {message}", &content[span.start..span.end])
                }
            }

            exit(1);
        }
    }
}
