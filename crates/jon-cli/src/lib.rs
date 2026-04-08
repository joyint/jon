use clap::Parser;

/// Jon - Natural language interface for Joy and Jot
#[derive(Parser)]
#[command(name = "jon", version, about)]
struct Cli {
    /// The query to process
    query: Option<String>,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.query {
        Some(query) => {
            println!("Jon received: {query}");
            println!("Tier 0 pattern router not yet implemented.");
        }
        None => {
            println!("Jon v{}", env!("CARGO_PKG_VERSION"));
            println!("Natural language interface for Joy and Jot.");
            println!();
            println!("Usage: jon \"what's my next task?\"");
        }
    }

    Ok(())
}
