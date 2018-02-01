extern crate image;
extern crate clap;

use image::*;

use std::fs::File;

use clap::{Arg, App};

fn main() {
    let matches =
        App::new("Rayer")
        .version("1.0")
        .arg(Arg::with_name("output")
             .long("output")
             .value_name("FILE")
             .required(true)
             .takes_value(true))
        .get_matches();
    let output = matches.value_of("output").unwrap();
    let height = 200;
    let width = 400;
    let mut buffer = ImageBuffer::new(width, height);

    for i in 0..width {
        for j in 0..height {
            let pixel =
                [((i as f32)/(width as f32)*255.99) as u8
                ,((j as f32)/(height as f32)*255.99) as u8
                ,(0.2*255.9) as u8
                ];
            buffer[(i,j)] = Rgb(pixel);
        }
    }

    let ref mut fout = File::create(output).unwrap();

    ImageRgb8(buffer).save(fout, PNG).unwrap();
}
