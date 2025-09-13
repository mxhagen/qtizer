use clap::{CommandFactory, Parser};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::colors::ColorFormat;

mod cli;
mod colors;
mod kmeans;

fn main() {
    let args = cli::Args::parse();
    cli::semantically_validate(&args);

    let seed = args.seed.unwrap_or_else(|| {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("you are a time traveler (system time < unix epoch)")
            .as_millis();

        // least significant 64 bits
        (millis & u64::MAX as u128) as u64
    });

    let mut context = kmeans::Context::new(seed);

    // TODO: use rgb only, when use-alpha is false
    //       with this refactor, also look at storing results & necessary config in
    //       `Context` in order to pass it uniformly to the various handlers

    // open file and parse image
    let img = image::open(args.file_path)
        .expect("failed to open image")
        .to_rgba8();

    // run kmeans
    let pixels = img.pixels().cloned().collect::<Vec<_>>();
    let (clusters, _assignments) = context.k_means(&pixels, args.number, args.iterations);

    // handle output
    match args.output.or(args.output_positional) {
        None => palette_handler(
            &clusters,
            &mut std::io::stdout(),
            args.alpha,
            &args.format.unwrap_or_default(),
        ),

        // TODO: implement image output
        // Some(output_file) if is_image_file_path(&output_file) => image_file_handler(&clusters, &assignments, output_file),
        Some(output_file) => {
            let mut file =
                std::fs::File::create(output_file).expect("failed to create output file");
            palette_handler(
                &clusters,
                &mut file,
                args.alpha,
                &args.format.unwrap_or_default(),
            );
        }
    }
}

fn palette_handler<W>(
    clusters: &[image::Rgba<u8>],
    writer: &mut W,
    alpha: bool,
    format: &ColorFormat,
) where
    W: std::io::Write,
{
    // sort colors by alpha and brightness
    let mut clusters = clusters.to_vec();
    clusters.sort_by(|a, b| {
        a[3].cmp(&b[3]) // alpha first
            .then_with(|| u32::cmp(&colors::brightness(b), &colors::brightness(a))) // then brightness
    });

    // output palette as hex #rrggbbaa
    // output with ansi escape codes for color preview in terminal
    for color in &clusters {
        ColorFormat::pretty_print_color_code(format, writer, color, alpha);
        writeln!(writer).expect("failed to write color to output");
    }
}
