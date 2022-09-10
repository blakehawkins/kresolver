use std::env::args;
use std::io::stdout;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub use kresolver::{Book, Ledger};

fn main() -> Result<()> {
    let path = args().nth(1).context("A csv path was not provided.")?;
    let path = PathBuf::try_from(path)?;
    Book::new()
        .resolve_all(Ledger::try_from(path)?)
        .write(stdout())?;

    Ok(())
}
