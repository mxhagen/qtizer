use crate::colors::ColorFormat;
use clap::*;

/// Quantization/palette-generation tool using k-means clustering on pixel dataI
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Input file path
    #[arg(index = 1, value_name = "input")]
    pub file_path: String,

    /// Number of colors to quantize to
    #[arg(short = 'k', default_value_t = 8, value_name = "count")]
    pub number: usize,

    /// Number of k-means iterations to perform
    #[arg(short = 'n', default_value_t = 5, value_name = "count")]
    pub iterations: usize,

    /// Include alpha channel
    #[arg(short = 'a', long = "with-alpha", default_value_t = false)]
    pub alpha: bool,

    /// Optional RNG seed for reproducible results
    #[arg(short = 's', long = "seed", value_name = "number")]
    pub seed: Option<u64>,

    /// Output file path
    /// - If not provided, outputs to stdout
    // TODO: implement image output
    // /// - With image file extensions, outputs an image file
    #[arg(
        short = 'o',
        long = "output",
        conflicts_with = "output_positional",
        value_name = "output",
        verbatim_doc_comment
    )]
    pub output: Option<String>,

    /// Output file path
    #[arg(index = 2, conflicts_with = "output", value_name = "output")]
    pub output_positional: Option<String>,

    /// Palette output format
    #[arg(
        short = 'f',
        long = "format",
        default_value = "hex",
        value_name = "fmt"
    )]
    pub format: Option<ColorFormat>,
    // TODO: implement multi-threading
    // /// Number of workers to use [default: core count]
    // #[arg(short = 'j', long = "jobs",
    //       value_parser = clap::value_parser!(u32).range(1..),
    //       value_name = "count")]
    // pub jobs: usize,
}

/// semantic validation of arguments
pub fn semantically_validate(args: &Args) {
    if args.format.is_some()
        && (args.output.clone())
            .or(args.output_positional.clone())
            .is_some_and(is_image_file_path)
    {
        Args::command()
            .error(
                clap::error::ErrorKind::ArgumentConflict,
                "cannot specify color format when outputting an image file.",
            )
            .exit();
    }
}

// TODO: implement image output
pub fn is_image_file_path<P>(file: P) -> bool
where
    P: AsRef<std::path::Path>,
{
    file.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| {
            matches!(
                e.to_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "bmp" | "tiff" | "ppm"
            )
        })
}

// TODO: implement static logger functionality for parsed arguments
