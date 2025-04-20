use clap::Parser;
use cli::CompilerArgs;

mod cli;
mod optimizer;
mod parser;
mod wbf;

#[cfg(test)]
mod tests;

fn main() -> Result<(), String> {
    let args = CompilerArgs::parse();
    args.print_status();

    args.run().map(|_| ())
}
