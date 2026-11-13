use clap::Parser;

/// Ordo — sort files into folders using simple rules.
#[derive(Parser, Debug)]
#[command(name = "ordo", version = ordo_core::VERSION, about)]
struct Cli {}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    Ok(())
}
