#![feature(test)]
extern crate clap;
extern crate euclid;
extern crate image;
extern crate num_traits;
extern crate palette;
extern crate rand;
extern crate test;

use clap::{Arg, App};
use euclid::*;
use num_traits::Float;
use palette::*;
use rand::Rng;
use std::fs::File;

mod camera;
mod color;
mod hitable;
mod ray;

use hitable::hitable_list::*;
use hitable::class::*;
use hitable::sphere::*;

fn rand_in_unit_sphere<R, T>(rng: &mut R) -> Vector3D<T>
where R: rand::Rng
    , T: Float + rand::distributions::range::SampleRange
{
    let mut p: Vector3D<T>;
    let mut gen_component = || rng.gen_range(-T::one(), T::one());
    while {
        p = Vector3D::new(gen_component(), gen_component(), gen_component());
        p.dot(p) >= T::one()
    } {}
    p
}

fn color<R: rand::Rng>(rng: &mut R, r: ray::Ray<f32>, world: &Hitable<f32>) -> Rgb<f32> {
    let rec = world.hit(r, 0.001, std::f32::MAX);
    match rec {
        Some(rec) => {
            let target = rec.p + rec.normal + rand_in_unit_sphere(rng);
            color(rng, ray::Ray::new(rec.p, target-rec.p), world)*0.5
        },
        None => {
            let unit_direction = r.direction.normalize();
            let t = (unit_direction.y + 1.0)*0.5;
            Rgb::new(1.0, 1.0, 1.0)*(1.0-t) + Rgb::new(0.5, 0.7, 1.0)*t
        }
    }
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
    let num_samples = 100;

    let mut buffer = image::ImageBuffer::new(width, height);

    let spheres: Vec<Sphere<f32>> = vec![
        Sphere::new(Point3D::new(0.0, 0.0, -1.0), 0.5),
        Sphere::new(Point3D::new(0.0, -100.5, -1.0), 100.0)
    ];
    let list: Vec<&Hitable<f32>> = spheres.iter().map(|sphere| sphere as &Hitable<f32>).collect();
    let world = HitableList(list.as_ref());
    let cam = camera::Camera::default();

    let mut rng = rand::thread_rng();
    for i in 0..width {
        for j in 0..height {
            let mut col: Rgb<f32> = Rgb::new(0.0, 0.0, 0.0);
            for _ in 0..num_samples {
                let u = ((i as f32) + rng.next_f32()) / (width as f32);
                let v = ((j as f32) + rng.next_f32()) / (height as f32);
                let r = cam.get_ray(u, v);
                col = col + color(&mut rng, r, &world);
            }
            let pixel =
                [(col.red/(num_samples as f32)*255.99) as u8
                ,(col.green/(num_samples as f32)*255.99) as u8
                ,(col.blue/(num_samples as f32)*255.9) as u8
                ];
            buffer[(i,height-j-1)] = image::Rgb(pixel);
        }
    }

    let ref mut fout = File::create(output).unwrap();

    image::ImageRgb8(buffer).save(fout, image::PNG).unwrap();
}
