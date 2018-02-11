#![feature(test)]
#![feature(const_fn)]
extern crate clap;
extern crate decorum;
extern crate euclid;
extern crate image;
extern crate num_traits;
extern crate palette;
extern crate rand;
extern crate test;

use std::sync::Arc;
use clap::{Arg, App};
use euclid::*;
use palette::*;
use palette::pixel::Srgb;
use palette::white_point::D65;
use std::fs::File;

mod camera;
mod color;
mod hitable;
mod material;
mod random;
mod ray;
mod types;

use color::HasReflectance;
use hitable::Hitable;
use hitable::bvh::*;
use hitable::sphere::*;
use material::*;
use random::*;
use types::*;

fn color<T: CoordinateBase>(r: ray::Ray<T>, world: &Hitable<T>) -> Xyz<D65, f32> {
    let refl = reflectance(r, world);
    color::xyz_from_wavelength(r.wl) * refl
}

fn reflectance<T: CoordinateBase>(r: ray::Ray<T>, world: &Hitable<T>) -> f32 {
    let mut r = r;
    let mut res = 0.0;
    let mut attenuation_acc = 1.0;
    for _ in 0..20 {
        let rec = world.hit(r, T::epsilon(), T::max_value());
        match rec {
            Some(rec) => {
                let mat_res = rec.material.scatter(r, rec);
                res += mat_res.emittance*attenuation_acc;
                match mat_res.reflection {
                    None => { return res; },
                    Some((attenuation, ray)) => {
                        r = ray;
                        attenuation_acc *= attenuation;
                    }
                }
            },
            None => {
                let unit_direction = r.direction.normalize();
                let t: f32 = (unit_direction.y.to_f32().unwrap() + 1.0)*0.5;
                let rgb = Rgb::new(1.0, 1.0, 1.0)*(1.0-t) + Rgb::new(0.5, 0.7, 1.0)*t;
                res += rgb.reflect(r.wl)*attenuation_acc;
                return res;
            }
        }
    }
    return res;
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

    let height = 600;
    let width = 800;
    let num_samples = 10;

    let mut buffer = image::ImageBuffer::new(width, height);

    let mat1 = Arc::new(Lambertian::new(Rgb::new(0.1, 0.2, 0.5)));
    let mat2 = Arc::new(Lambertian::new(Rgb::new(0.8, 0.8, 0.0)));
    let mat3 = Arc::new(Metal::new(Rgb::new(0.8, 0.6, 0.2), 1.0));
    let mat4 = Arc::new(Dielectric::SF11);
    let mut list: Vec<Arc<Hitable<f32>>> = vec![
        Arc::new(Sphere::new(Point3D::new(0.0, 0.0, -1.0), 0.5, mat1)),
        Arc::new(Sphere::new(Point3D::new(0.0, -100.5, -1.0), 100.0, mat2)),
        Arc::new(Sphere::new(Point3D::new(1.0, 0.0, -1.0), 0.5, mat3)),
        Arc::new(Sphere::new(Point3D::new(-1.0, 0.0, -1.0), 0.5, mat4.clone())),
        Arc::new(Sphere::new(Point3D::new(-1.25, 0.0, -1.0), -0.20, mat4.clone())),
        Arc::new(Sphere::new(Point3D::new(-0.75, 0.0, -1.0), -0.20, mat4)),
    ];
    let world = BVH::initialize(list.as_mut_slice());
    let look_from = Point3D::new(-4.0, 0.7, 3.0);
    let look_at = Point3D::new(-1.0, 0.0, -1.0);
    let focus_dist = (look_from-look_at).length();
    let aperture = 0.1;
    let up = Vector3D::new(0.0, 1.0, 0.0);

    let cam = camera::Camera::new(look_from, look_at, up, 70.0, width as f32/height as f32, aperture, focus_dist);

    let wl_low = 390.0;
    let wl_high = 700.0;
    let wl_span = wl_high-wl_low;
    for i in 0..width {
        for j in 0..height {
            let mut col: Xyz<D65, f32> = Xyz::new(0.0, 0.0, 0.0);
            let mut wl = gen_range(wl_low, wl_high);
            for _ in 0..num_samples {
                let u = ((i as f32) + next_f32()) / (width as f32);
                let v = ((j as f32) + next_f32()) / (height as f32);
                let r = cam.get_ray(u, v, wl);
                col = col + color(r, &world);
                wl += wl_span/(num_samples as f32);
                if wl>wl_high {
                    wl -= wl_span;
                }
            }
            let col = col*3.0;
            let col = (col.into_rgb()/(num_samples as f32)).clamp();
            let col = Srgb::from(col);
            let pixel =
                [(col.red*255.99) as u8
                ,(col.green*255.99) as u8
                ,(col.blue*255.9) as u8
                ];
            buffer[(i,height-j-1)] = image::Rgb(pixel);
        }
    }

    let ref mut fout = File::create(output).unwrap();

    image::ImageRgb8(buffer).save(fout, image::PNG).unwrap();
}
