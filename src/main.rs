mod cli;
mod core;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::parse_args();
    core::run(args)?;
    Ok(())
}
