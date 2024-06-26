mod dye;

use std::{collections::HashMap, fmt::Display, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::bail;
use dye::{Color, DyeCalcMode};
use itertools::Itertools;
use prettytable::{row, Table};
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// The CSV file to be read and analyzed
    pub file: PathBuf,

    /// Use the total item counts instead of missing, for example to have a global view even when halfway built already
    #[arg(long)]
    pub use_total: bool,

    /// List items in the file
    #[arg(long)]
    pub list: bool,

    /// Compute dye quantities for dyeable blocks
    #[arg(long, value_enum)]
    pub dye_calc: Option<DyeCalcMode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawMaterial {
    item: String,
    total: u32,
    missing: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Item {
    item: String,
    color: Option<Color>,
    count: u32,
}

impl Item {
    fn from_raw(raw: RawMaterial, use_total: bool) -> Self {
        let (color, item) = Color::split_color(&raw.item);
        let count = if use_total { raw.total } else { raw.missing };

        Self {
            item: item.to_string(),
            color,
            count,
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.color {
            Some(c) => write!(f, "{} {}", c, self.item),
            None => write!(f, "{}", self.item),
        }
    }
}

fn main() -> color_eyre::eyre::Result<()> {
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
        let mut items = Vec::new();

        for line in reader.deserialize() {
            let line: RawMaterial = line?;
            let item = Item::from_raw(line, args.use_total);

            if item.count != 0 {
                items.push(item);
            }
        }

        items
    };

    if args.list {
        print_table(
            ("Item", "Quantity"),
            items.iter().map(|i| (i, i.count)).collect(),
        );
    }

    if let Some(dye_calc) = args.dye_calc {
        let colors = dye::compute_colors(&items);

        print_table(("Color", "Count"), sort_map(&colors));

        if matches!(
            dye_calc,
            DyeCalcMode::Primary | DyeCalcMode::PrimaryAndQuasi
        ) {
            let dyes = dye::compute_dye_ingredients(colors, dye_calc);

            print_table(("Dye", "Quantity"), sort_map(&dyes));
        }
    }

    Ok(())
}

fn sort_map<K, V: Ord>(map: &HashMap<K, V>) -> Vec<(&K, &V)> {
    map.iter().sorted_by_key(|(_, v)| *v).rev().collect()
}

fn print_table<K: Display, V: Display>(header: (&str, &str), map: Vec<(K, V)>) {
    let mut table = Table::new();
    table.add_row(row![header.0, header.1]);

    for (key, value) in map {
        table.add_row(row![key, value]);
    }

    table.printstd();
}
