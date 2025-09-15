use clap::*;
use image::*;
use std::time::{SystemTime, UNIX_EPOCH};

mod cli;
mod colors;
mod kmeans;

use crate::colors::*;

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

    // open file and parse image
    let img = image::open(args.file_path).expect("failed to open image");

    let pixels = match args.alpha {
        true => img
            .to_rgba8()
            .pixels()
            .map(|p| Color {
                data: p.0.to_vec(),
                color_type: ColorType::Rgba8,
            })
            .collect::<Vec<_>>(),
        false => img
            .to_rgb8()
            .pixels()
            .map(|p| Color {
                data: p.0.to_vec(),
                color_type: ColorType::Rgb8,
            })
            .collect::<Vec<_>>(),
    };

    // run kmeans
    let (clusters, assignments) = context.k_means(&pixels, args.number, args.iterations);

    // handle output
    match args.output.or(args.output_positional) {
        None => palette_handler(
            &clusters,
            &mut std::io::stdout(),
            &args.format.unwrap_or_default(),
        ),

        Some(output_file) if ImageFormat::from_path(&output_file).is_ok() => {
            let (width, height) = img.dimensions();
            image_file_handler(width, height, &clusters, &assignments, output_file);
        }

        Some(output_file) => {
            let mut file =
                std::fs::File::create(output_file).expect("failed to create output file");
            palette_handler(&clusters, &mut file, &args.format.unwrap_or_default());
        }
    }
}

/// handle palette output to terminal or file
fn palette_handler<W>(clusters: &[Color], writer: &mut W, format: &ColorCodeFormat)
where
    W: std::io::Write,
{
    // sort colors by alpha and brightness
    let mut clusters = clusters.to_vec();
    clusters.sort_by(|x, y| {
        let (&[r_x, g_x, b_x], &[r_y, g_y, b_y]) = (&x.data[..3], &y.data[..3]) else {
            unreachable!("invalid color type. only rgb or rgba colors should ever be used here.");
        };
        u32::cmp(&colors::brightness(y), &colors::brightness(x)) // descending brightness
            .then_with(|| {
                u32::from_be_bytes([r_x, g_x, b_x, 0]).cmp(&u32::from_be_bytes([r_y, g_y, b_y, 0]))
            })
    });

    // output palette as hex #rrggbbaa
    // output with ansi escape codes for color preview in terminal
    for color in &clusters {
        ColorCodeFormat::pretty_print_color_code(format, writer, color);
        writeln!(writer).expect("failed to write color to output");
    }
}

/// handle image output to file
fn image_file_handler(
    width: u32,
    height: u32,
    clusters: &[Color],
    assignments: &[usize],
    output_file: String,
) {
    // create new image by replacing each pixel with its cluster center
    let quantized = assignments
        .iter()
        .flat_map(|&i| &clusters[i].data)
        .copied()
        .collect::<Vec<_>>();

    let status = match clusters.first() {
        Some(c) if c.color_type == ColorType::Rgba8 => {
            let img = ImageBuffer::from_vec(width, height, quantized);
            let img: ImageBuffer<Rgba<u8>, _> = img.expect("failed to create quantized image");
            img.save(&output_file)
        }
        Some(c) if c.color_type == ColorType::Rgb8 => {
            let img = ImageBuffer::from_vec(width, height, quantized);
            let img: ImageBuffer<Rgb<u8>, _> = img.expect("failed to create quantized image");
            img.save(&output_file)
        }
        _ => {
            cli::err_exit(
                clap::error::ErrorKind::InvalidValue,
                "unexpected error - this is a bug".to_string(),
            );
            unreachable!("should have exited");
        }
    };

    // TODO: better errors handling logger
    // save image with inferred format
    match status {
        Ok(_) => println!("saved quantized image to {output_file}"),
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
