use clap::*;
use image::*;
use std::time::{SystemTime, UNIX_EPOCH};

mod cli;
mod colors;
mod kmeans;

use crate::colors::ColorFormat;

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
    let (clusters, assignments) = context.k_means(&pixels, args.number, args.iterations);

    // handle output
    match args.output.or(args.output_positional) {
        None => palette_handler(
            &clusters,
            &mut std::io::stdout(),
            args.alpha,
            &args.format.unwrap_or_default(),
        ),

        Some(output_file) if ImageFormat::from_path(&output_file).is_ok() => {
            image_file_handler(&img, &clusters, &assignments, args.alpha, output_file)
        }

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

/// handle palette output to terminal or file
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

/// handle image output to file
fn image_file_handler(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    clusters: &[image::Rgba<u8>],
    assignments: &[usize],
    alpha: bool,
    output_file: String,
) {
    let (width, height) = img.dimensions();

    // create new image by replacing each pixel with its cluster center
    let quantized = assignments
        .iter()
        .flat_map(|&i| &clusters[i].0[..if alpha { 4 } else { 3 }])
        .copied()
        .collect::<Vec<_>>();

    let status = match alpha {
        true => {
            let img = ImageBuffer::from_vec(width, height, quantized);
            let img: ImageBuffer<Rgba<u8>, _> = img.expect("failed to create quantized image");
            img.save(&output_file)
        }
        false => {
            let img = ImageBuffer::from_vec(width, height, quantized);
            let img: ImageBuffer<Rgb<u8>, _> = img.expect("failed to create quantized image");
            img.save(&output_file)
        }
    };

    // TODO: better errors handling logger
    // save image with inferred format
    match status {
        Ok(_) => println!("saved quantized image to {}", output_file),
        Err(err) => {
            // errors here are unexpected, since extension alpha-capability
            // is validated in `cli::semantically_validate`
            cli::err_exit(
                clap::error::ErrorKind::InvalidValue,
                "unexpectedly failed to save quantized image.\n".to_string()
                    + "try checking the output file format. (does it support alpha?)\n"
                    + &format!("    ({err})"),
            );
        }
    }
}
