use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{bail, Result};
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// The CSV file to be read and analyzed
    file: PathBuf,

    /// Use the total item counts instead of missing, for example to have a global view even when halfway built already
    #[arg(long)]
    use_total: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MaterialLine {
    item: String,
    total: u32,
    missing: u32,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let path = args.file.canonicalize()?;
    let mut reader = csv::Reader::from_path(&path)?;

    let headers = reader.headers()?;
    let is_file_valid = headers.get(0).is_some_and(|s| s == "Item");

    if !is_file_valid {
        bail!("You need to export the material list in CSV format! Shift+Click the export button in litematica to generate a CSV file");
    }

    let items = {
        let mut items = HashMap::new();

        for line in reader.deserialize() {
            let line: MaterialLine = line?;

            let count = if args.use_total {
                line.total
            } else {
                line.missing
            };

            if count != 0 {
                items.insert(line.item, count);
            }
        }

        items
    };

    println!("{:?}", items);

    Ok(())
}
