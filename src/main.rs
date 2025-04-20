use clap::Parser;
use optimizer::Optimizer;
use parser::{
    ast::{BInstr, Reconstruct},
    parse_program,
};
use std::path::PathBuf;
use wbf::WBFEmitter;

mod optimizer;
mod parser;
mod wbf;

#[cfg(test)]
mod tests;

/// WORN (Write Once, Run Nowhere):
/// The "ultimate" Brainfuck emitter/compiler/optimizer
#[derive(Parser, Debug)]
#[command(name = "worn", author = "michael-0acf4", about)]
struct CompilerArgs {
    /// Input source file
    #[arg()]
    file: PathBuf,
    /// Set the output file
    #[arg(short)]
    output: Option<PathBuf>,
    /// Custom optimization level
    #[arg(short = 'O', long, default_value = "3")]
    optimize: Option<u8>,
    #[arg(short, long)]
    print: bool,
}

impl CompilerArgs {
    pub fn run(self) -> Result<Vec<BInstr>, String> {
        let content = std::fs::read_to_string(self.file).expect("Unable to read file");
        parse_program(&content).and_then(|program| {
            let mut emitter = WBFEmitter::new(program);
            emitter.compile().map_err(|e| e.to_string())?;

            let mut program = emitter.finalize()?;
            let mut program_str = program.reconstruct();
            let og_count = program_str.len();

            if let Some(level) = self.optimize {
                let opt = Optimizer { level };
                program = opt.apply(program);
                program_str = program
                    .iter()
                    .map(|bi| bi.reconstruct())
                    .collect::<Vec<_>>()
                    .concat();
                let opt_count = program_str.len();
                println!("From {og_count} to {opt_count} characters.");
            }

            if let Some(output) = self.output {
                std::fs::write(output, &program_str).expect("Failed writing into output file");
            }

            if self.print {
                println!("\n{program_str}");
            }

            Ok(program)
        })
    }

    pub fn print_status(&self) {
        println!("Target: {}", self.file.display());
        println!(
            "Output: {}",
            self.output
                .clone()
                .map(|f| f.display().to_string())
                .unwrap_or("<stdout>".to_owned())
        );
        println!("Opt level: {}", self.optimize.clone().unwrap_or(3));
        println!()
    }
}

fn main() -> Result<(), String> {
    let args = CompilerArgs::parse();
    args.print_status();

    args.run().map(|_| ())
}
