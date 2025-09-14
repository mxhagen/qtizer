use image::Rgba;

use crate::kmeans::Kmeansable;

// TODO: implement k-means for `Rgb<u8>` pixel format
//       for slightly less mem-usage and faster calculation

impl Kmeansable for Rgba<u8> {
    type Sum = Rgba<u32>;

    /// black, transparent
    fn zero() -> Self::Sum {
        Rgba([0, 0, 0, 0])
    }

    /// euclidean distance between two rgba colors
    fn distance(&self, other: &Self) -> f64 {
        let dr = (self[0] as f64) - (other[0] as f64);
        let dg = (self[1] as f64) - (other[1] as f64);
        let db = (self[2] as f64) - (other[2] as f64);
        let da = (self[3] as f64) - (other[3] as f64);
        (dr * dr + dg * dg + db * db + da * da).sqrt()
    }

    /// add two rgba colors, returning a u32 sum to avoid overflow
    fn add(sum: &Self::Sum, other: &Self) -> Self::Sum {
        Rgba([
            sum[0] + other[0] as u32,
            sum[1] + other[1] as u32,
            sum[2] + other[2] as u32,
            sum[3] + other[3] as u32,
        ])
    }

    /// calculate the mean of a sum of rgba colors and a count
    fn div(sum: &Self::Sum, count: usize) -> Self {
        if count == 0 {
            // TODO: better error handling via static logger
            panic!("tried to calculate mean of 0 colors (division by zero)");
        }

        Rgba([
            (sum[0] / count as u32) as u8,
            (sum[1] / count as u32) as u8,
            (sum[2] / count as u32) as u8,
            (sum[3] / count as u32) as u8,
        ])
    }
}

/// calculate the rgba brightness (luminance)
pub fn brightness(&Rgba([r, g, b, _]): &Rgba<u8>) -> u32 {
    ((0.299 * r as f32) + (0.587 * g as f32) + (0.114 * b as f32)) as u32
}

/// color code output format
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum ColorFormat {
    /// `#rrggbb` or `#rrggbbaa`
    #[default]
    Hex,
    /// `rgb(r, g, b)` or `rgba(r, g, b, a)`
    Rgb,
}
use ColorFormat::*;

impl ColorFormat {
    /// pretty print a color code in the format
    /// when writing to terminals, uses ansi escape codes for color preview
    pub fn pretty_print_color_code<W>(
        format: &ColorFormat,
        writer: &mut W,
        color: &image::Rgba<u8>,
        with_alpha: bool,
    ) where
        W: std::io::Write,
    {
        match format {
            Hex => Self::colored_with_format(writer, color, with_alpha, Self::hex_color_code),
            Rgb => Self::colored_with_format(writer, color, with_alpha, Self::rgb_color_code),
        }
    }

    /// pretty print wrapper that colors output
    /// given a callback providing the actual color formatting
    fn colored_with_format<W>(
        writer: &mut W,
        color: &image::Rgba<u8>,
        with_alpha: bool,
        callback: fn(&mut W, &image::Rgba<u8>, bool),
    ) where
        W: std::io::Write,
    {
        use std::io::IsTerminal;
        let colorize = std::io::stdout().is_terminal();

        if !colorize {
            // just print formatted color, no ansi codes
            return callback(writer, color, with_alpha);
        }

        // ensure text has enough contrast to colored background
        match brightness(&color) {
            ..128 => write!(writer, "\x1b[38;2;255;255;255m"), // dark  => white text
            _ => write!(writer, "\x1b[38;2;0;0;0m"),           // light => black text
        }
        .expect("failed to write output");

        // print ansi codes for colored background
        write!(writer, "\x1b[48;2;{};{};{}m", color[0], color[1], color[2],)
            .expect("failed to write output");

        // call the actual color printing function
        callback(writer, color, with_alpha);

        // reset colors
        write!(writer, "\x1b[0m").expect("failed to write output");
    }

    /// print uncolored hex color code, with optional alpha
    fn hex_color_code<W>(writer: &mut W, color: &image::Rgba<u8>, with_alpha: bool)
    where
        W: std::io::Write,
    {
        let Rgba([r, g, b, a]) = color;

        write!(writer, "#{:02x}{:02x}{:02x}", r, g, b).expect("failed to write output");

        if with_alpha {
            write!(writer, "{:02x}", a).expect("failed to write output");
        }
    }

    /// print uncolored rgb color code, with optional alpha
    fn rgb_color_code<W>(writer: &mut W, color: &image::Rgba<u8>, with_alpha: bool)
    where
        W: std::io::Write,
    {
        let Rgba([r, g, b, a]) = color;

        match with_alpha {
            true => write!(writer, "rgba({}, {}, {}, {})", r, g, b, a),
            false => write!(writer, "rgb({}, {}, {})", r, g, b),
        }
        .expect("failed to write output")
    }
}
