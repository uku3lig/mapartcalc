use std::{collections::HashMap, fmt::Display};

use clap::ValueEnum;
use itertools::Itertools;
use owo_colors::OwoColorize;

use crate::Item;

pub const DYEABLE: [&str; 4] = [
    "Terracotta",
    "Concrete Powder",
    "Stained Glass",
    "Stained Glass Pane",
];

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DyeCalcMode {
    /// Don't compute any dye crafts, just output the colors
    NoCalc,
    /// Compute all the primary dyes to craft all the colors
    Primary,
    /// Compute all the primary and quasi-primary dyes to craft all the colors
    PrimaryAndQuasi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    LightGray,
    Gray,
    Black,
    Brown,
    Red,
    Orange,
    Yellow,
    Lime,
    Green,
    Cyan,
    LightBlue,
    Blue,
    Purple,
    Magenta,
    Pink,
}

impl Color {
    pub fn parse(color: &str) -> Option<Self> {
        match color {
            "White" => Some(Color::White),
            "Light Gray" => Some(Color::LightGray),
            "Gray" => Some(Color::Gray),
            "Black" => Some(Color::Black),
            "Brown" => Some(Color::Brown),
            "Red" => Some(Color::Red),
            "Orange" => Some(Color::Orange),
            "Yellow" => Some(Color::Yellow),
            "Lime" => Some(Color::Lime),
            "Green" => Some(Color::Green),
            "Cyan" => Some(Color::Cyan),
            "Light Blue" => Some(Color::LightBlue),
            "Blue" => Some(Color::Blue),
            "Purple" => Some(Color::Purple),
            "Magenta" => Some(Color::Magenta),
            "Pink" => Some(Color::Pink),
            _ => None,
        }
    }

    pub fn split_color(item: &str) -> (Option<Self>, &str) {
        let Some((first, second)) = item.split_once(" ") else {
            return (None, item);
        };

        if first == "Light" {
            let (second, third) = second.split_once(" ").unwrap();
            let color = Self::parse(&format!("Light {}", second));
            (color, color.map_or(item, |_| third))
        } else {
            let color = Self::parse(first);
            (color, color.map_or(item, |_| second))
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "{}", "White".fg_rgb::<0xF9, 0xFF, 0xFE>()),
            Color::LightGray => write!(f, "{}", "Light Gray".fg_rgb::<0x9D, 0x9D, 0x97>()),
            Color::Gray => write!(f, "{}", "Gray".fg_rgb::<0x47, 0x4F, 0x52>()),
            Color::Black => write!(f, "{}", "Black".fg_rgb::<0x1D, 0x1D, 0x21>()),
            Color::Brown => write!(f, "{}", "Brown".fg_rgb::<0x83, 0x54, 0x32>()),
            Color::Red => write!(f, "{}", "Red".fg_rgb::<0xB0, 0x2E, 0x26>()),
            Color::Orange => write!(f, "{}", "Orange".fg_rgb::<0xF9, 0x80, 0x1D>()),
            Color::Yellow => write!(f, "{}", "Yellow".fg_rgb::<0xFE, 0xD8, 0x3D>()),
            Color::Lime => write!(f, "{}", "Lime".fg_rgb::<0x80, 0xC7, 0x1F>()),
            Color::Green => write!(f, "{}", "Green".fg_rgb::<0x5E, 0x7C, 0x16>()),
            Color::Cyan => write!(f, "{}", "Cyan".fg_rgb::<0x16, 0x9C, 0x9C>()),
            Color::LightBlue => write!(f, "{}", "Light Blue".fg_rgb::<0x3A, 0xB3, 0xDA>()),
            Color::Blue => write!(f, "{}", "Blue".fg_rgb::<0x3C, 0x44, 0xAA>()),
            Color::Purple => write!(f, "{}", "Purple".fg_rgb::<0x89, 0x32, 0xB8>()),
            Color::Magenta => write!(f, "{}", "Magenta".fg_rgb::<0xC7, 0x4E, 0xBD>()),
            Color::Pink => write!(f, "{}", "Pink".fg_rgb::<0xF3, 0x8B, 0xAA>()),
        }
    }
}

pub struct Addition {
    result: Color,
    addends: &'static [Color],
}

impl Addition {
    pub const TERTIARY: [Self; 3] = [
        Self {
            result: Color::Gray,
            addends: &[Color::White, Color::Black],
        },
        Self {
            result: Color::Purple,
            addends: &[Color::Red, Color::Blue],
        },
        Self {
            result: Color::Cyan,
            addends: &[Color::Green, Color::Blue],
        },
    ];

    pub const QUASI: [Self; 6] = [
        Self {
            result: Color::LightBlue,
            addends: &[Color::Blue, Color::White],
        },
        Self {
            result: Color::LightGray,
            addends: &[Color::Black, Color::White, Color::White],
        },
        Self {
            result: Color::Lime,
            addends: &[Color::Green, Color::White],
        },
        Self {
            result: Color::Magenta,
            addends: &[Color::Blue, Color::White, Color::Red, Color::Red],
        },
        Self {
            result: Color::Orange,
            addends: &[Color::Red, Color::Yellow],
        },
        Self {
            result: Color::Pink,
            addends: &[Color::Red, Color::White],
        },
    ];
}

pub fn compute_colors(items: &[Item]) -> HashMap<Color, u32> {
    items
        .iter()
        .filter(|item| DYEABLE.contains(&item.item.as_str()))
        .filter_map(|item| item.color.map(|c| (c, item.count.div_ceil(8))))
        .into_grouping_map()
        .sum()
}

pub fn compute_dye_ingredients(
    mut colors: HashMap<Color, u32>,
    mode: DyeCalcMode,
) -> HashMap<Color, u32> {
    if matches!(mode, DyeCalcMode::NoCalc) {
        return HashMap::new();
    }

    compute_additions(&mut colors, &Addition::TERTIARY);

    if matches!(mode, DyeCalcMode::Primary) {
        compute_additions(&mut colors, &Addition::QUASI);
    }

    colors
}

/// Replaces the dyes specified in each addition with the correct amount of crafting ingredients
///
/// ex: 3 Lime -> 2 Green, 2 White
fn compute_additions(map: &mut HashMap<Color, u32>, additions: &[Addition]) {
    for addition in additions {
        if let Some(count) = map.remove(&addition.result) {
            let len = addition.addends.len();
            let count = count.div_ceil(len as u32);

            for addend in addition.addends {
                *map.entry(*addend).or_default() += count;
            }
        }
    }
}
