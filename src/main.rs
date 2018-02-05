#![feature(test)]
extern crate num_traits;
extern crate euclid;
extern crate image;
extern crate clap;
extern crate palette;
extern crate test;

use euclid::*;
use palette::*;
use std::fs::File;
use clap::{Arg, App};
use num_traits::float::*;

mod ray;
mod color;

use ray::Ray;

fn color(r: ray::Ray<f32>) -> Rgb<f32> {
    let unit_direction = r.direction.normalize();
    let t = (unit_direction.y + 1.0)*0.5;
    Rgb::new(1.0, 1.0, 1.0)*(1.0-t) + Rgb::new(0.5, 0.7, 1.0)*t
}

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
    let mut buffer = image::ImageBuffer::new(width, height);

    let lower_left_corner = Point3D::new(-2.0, -1.0, -1.0);
    let horizontal = Vector3D::new(4.0, 0.0, 0.0);
    let vertical = Vector3D::new(0.0, 2.0, 0.0);
    let origin = Point3D::new(0.0, 0.0, 0.0);
    for i in 0..width {
        for j in 0..height {
            let u = (i as f32) / (width as f32);
            let v = (j as f32) / (height as f32);
            let r = Ray::new(origin, origin - (lower_left_corner + horizontal*u + vertical*v));
            let col = color(r);
            let pixel =
                [(col.red*255.99) as u8
                ,(col.green*255.99) as u8
                ,(col.blue*255.9) as u8
                ];
            buffer[(i,j)] = image::Rgb(pixel);
        }
    }

    let ref mut fout = File::create(output).unwrap();

    image::ImageRgb8(buffer).save(fout, image::PNG).unwrap();
}
