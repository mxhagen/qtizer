use clap::*;
use image::*;

use crate::colors::ColorCodeFormat;

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
    /// - With image file extensions, outputs an image file
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
    #[arg(short = 'f', long = "format", value_name = "fmt")]
    pub format: Option<ColorCodeFormat>,

    // TODO: add input alpha policy for opaque output images
    // /// Transparency policy when input has alpha but output does not
    // #[arg(short = 'p', long = "alpha-policy", value_name = "policy",)]
    // pub alpha_policy: AlphaPolicy,

    // TODO: implement multi-threading
    // /// Number of workers to use [default: core count]
    // #[arg(short = 'j', long = "jobs",
    //       value_parser = clap::value_parser!(u32).range(1..),
    //       value_name = "count")]
    // pub jobs: usize,
}

/// semantic validation of arguments
/// - `--format` cannot be specified when outputting an image file
/// - some image formats do not support alpha (eg. jpg)
pub fn semantically_validate(args: &Args) {
    // check if `--format` is specified AND output has image file extension
    if args.format.is_some()
        && (args.output.clone())
            .or(args.output_positional.clone())
            .is_some_and(|p| ImageFormat::from_path(p).is_ok())
    {
        err_exit(
            clap::error::ErrorKind::ArgumentConflict,
            "cannot specify color-code format when outputting an image file.",
        );
    }

    // check if output image format supports alpha channel
    let output_opt = args.output.clone().or(args.output_positional.clone());
    if args.alpha && output_opt.is_some() {
        let output_file = output_opt.unwrap();
        let filetype = ImageFormat::from_path(&output_file);

        use ImageFormat::*;
        if matches!(filetype, Ok(Jpeg | Bmp | Pnm | Tiff)) {
            err_exit(
                clap::error::ErrorKind::ArgumentConflict,
                format!(
                    "the `{:?}` image format does not support alpha.",
                    filetype.unwrap(),
                ),
            );
        }
    }
}

/// shorthand for `Args::command().error(...).exit()`
pub fn err_exit(kind: clap::error::ErrorKind, message: impl std::fmt::Display) {
    Args::command().error(kind, message).exit()
}

// TODO: implement static logger functionality for parsed arguments
