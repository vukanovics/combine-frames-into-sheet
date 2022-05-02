use clap::{Arg, Command};
use image::{ImageBuffer, DynamicImage, imageops, io::Reader};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;

fn main() {
    let matches = Command::new("Combine frames into sheet")
        .version("0.1.0")
        .arg(Arg::new("output")
             .short('o')
             .long("output")
             .takes_value(true)
             .required(true)
             .help("Output file"))
        .arg(Arg::new("rows")
             .short('r')
             .long("rows")
             .takes_value(true)
             .required(true)
             .help("How many resulting rows should the output sheet have"))
        .arg(Arg::new("columns")
             .short('c')
             .long("columns")
             .takes_value(true)
             .required(true)
             .help("How many resulting columns should the output sheet have"))
        .arg(Arg::new("inputs")
             .short('i')
             .long("inputs")
             .takes_value(true)
             .multiple_values(true)
             .required(true)
             .help("Input files"))
        .get_matches();

    let output = matches.value_of("output").expect("No output specified");
    let rows = matches.value_of("rows").map(| rows | rows.parse::<u32>().unwrap()).unwrap_or(1);
    let columns = matches.value_of("columns").map(| columns | columns.parse::<u32>().unwrap()).unwrap_or(1);
    let inputs: Vec::<&str> = matches.values_of("inputs").expect("No input files specified").collect();

    let bar_style = ProgressStyle::default_bar().template("[{spinner}] [{pos}]/[{len}] {msg}");
    let spinner_style = ProgressStyle::default_bar().template("[{spinner}] {msg}");

    let loading_progress = ProgressBar::new(inputs.len() as u64).with_message("Loading input images").with_style(bar_style.clone());

    let input_files: Vec::<DynamicImage> = inputs.par_iter().progress_with(loading_progress)
        .map(|input| -> DynamicImage {
            Reader::open(input)
                .expect("Failed to open file.")
                .decode()
                .expect("Invalid image file.")
        }).collect();

    let mut largest_width = 0 as i64;
    let mut largest_height = 0 as i64;

    for input in &input_files {
        largest_width = std::cmp::max(input.width() as i64, largest_width);
        largest_height = std::cmp::max(input.width() as i64, largest_height);
    }

    let output_height = largest_height * rows as i64;
    let output_width = largest_width * columns as i64;

    let mut output_image = ImageBuffer::new(output_width as u32, output_height as u32);

    let mut x = 0 as i64;
    let mut y = 0 as i64;

    let merging_progress = ProgressBar::new(input_files.len() as u64).with_message("Merging images").with_style(bar_style);

    for input in &input_files {
        imageops::replace(&mut output_image, input, x * largest_width, y * largest_height);
        merging_progress.inc(1);
        x = x + 1;
        if x == columns as i64 {
            y = y + 1;
            x = 0;
        }
    }

    merging_progress.finish_and_clear();

    let saving_progress = ProgressBar::new_spinner().with_message("Writing output").with_style(spinner_style);
    saving_progress.enable_steady_tick(15);

    output_image.save(output).unwrap();

    saving_progress.finish_with_message("Done!");
}
