use image::*;

use crate::kmeans::Kmeansable;

/// marker trait for usable color types
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Color {
    pub color_type: ColorType,
    pub data: Vec<u8>,
}

impl Kmeansable for Color {
    type Sum = Vec<u32>;

    fn zero() -> Self::Sum {
        vec![0; 4]
    }

    fn distance(&self, other: &Self) -> f64 {
        self.data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| ((*a as f64) - (*b as f64)).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    fn add(sum: &Self::Sum, other: &Self) -> Self::Sum {
        sum.iter()
            .zip(&other.data)
            .map(|(a, b)| a + *b as u32)
            .collect()
    }

    fn div(sum: &Self::Sum, count: usize) -> Self {
        let data = sum
            .iter()
            .map(|v| (v / count as u32) as u8)
            .collect::<Vec<u8>>();

        Color {
            color_type: match data.len() {
                3 => ColorType::Rgb8,
                4 => ColorType::Rgba8,
                _ => unreachable!(
                    "invalid color length. only rgb or rgba colors should ever be used here."
                ),
            },
            data,
        }
    }
}

/// calculate the rgba brightness (luminance)
pub fn brightness(color: &Color) -> u32 {
    let &[r, g, b, ..] = &color.data[..] else {
        unreachable!("invalid color type. only rgb or rgba colors should ever be used here.");
    };
    ((0.299 * r as f32) + (0.587 * g as f32) + (0.114 * b as f32)) as u32
}

/// color code output format
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum ColorCodeFormat {
    /// `#rrggbb` or `#rrggbbaa`
    #[default]
    Hex,
    /// `rgb(r, g, b)` or `rgba(r, g, b, a)`
    Rgb,
}

impl ColorCodeFormat {
    /// pretty print a color code in the format
    /// when writing to terminals, uses ansi escape codes for color preview
    pub fn pretty_print_color_code<W>(format: &ColorCodeFormat, writer: &mut W, color: &Color)
    where
        W: std::io::Write,
    {
        match format {
            ColorCodeFormat::Hex => Self::colored_with_format(writer, color, Self::hex_color_code),
            ColorCodeFormat::Rgb => Self::colored_with_format(writer, color, Self::rgb_color_code),
        }
    }

    /// pretty print wrapper that colors output
    /// given a callback providing the actual color formatting
    fn colored_with_format<W>(writer: &mut W, color: &Color, callback: fn(&mut W, &Color))
    where
        W: std::io::Write,
    {
        use std::io::IsTerminal;
        let colorize = std::io::stdout().is_terminal();

        if !colorize {
            // just print formatted color, no ansi codes
            return callback(writer, color);
        }

        // ensure text has enough contrast to colored background
        match brightness(color) {
            ..128 => write!(writer, "\x1b[38;2;255;255;255m"), // dark  => white text
            _ => write!(writer, "\x1b[38;2;0;0;0m"),           // light => black text
        }
        .expect("failed to write output");

        let &[r, g, b, ..] = &color.data[..] else {
            unreachable!("invalid color type. only rgb or rgba colors should ever be used here.");
        };

        // print ansi codes for colored background
        write!(writer, "\x1b[48;2;{r};{g};{b}m").expect("failed to write output");

        // call the actual color printing function
        callback(writer, color);

        // reset colors
        write!(writer, "\x1b[0m").expect("failed to write output");
    }

    /// print uncolored hex color code, with optional alpha
    fn hex_color_code<W>(writer: &mut W, color: &Color)
    where
        W: std::io::Write,
    {
        let c = &color.data;
        write!(writer, "#{:02x}{:02x}{:02x}", c[0], c[1], c[2]).expect("failed to write output");

        if color.color_type == ColorType::Rgba8 {
            write!(writer, "{:02x}", c[3]).expect("failed to write output");
        }
    }

    /// print uncolored rgb color code, with optional alpha
    fn rgb_color_code<W>(writer: &mut W, color: &Color)
    where
        W: std::io::Write,
    {
        let c = &color.data;
        match color.color_type {
            ColorType::Rgba8 => write!(writer, "rgba({}, {}, {}, {})", c[0], c[1], c[2], c[3]),
            ColorType::Rgb8 => write!(writer, "rgb({}, {}, {})", c[0], c[1], c[2]),
            _ => unreachable!(
                "invalid color type. only rgb or rgba colors should ever be used here."
            ),
        }
        .expect("failed to write output")
    }
}
