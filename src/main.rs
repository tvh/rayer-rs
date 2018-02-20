#![feature(const_fn)]
#![feature(fixed_size_array)]
#![feature(plugin)]
#![feature(slice_patterns)]
#![feature(test)]
#![plugin(quickcheck_macros)]
extern crate core;
extern crate clap;
extern crate cpuprofiler;
extern crate crossbeam_channel;
extern crate decorum;
extern crate euclid;
#[macro_use]
extern crate lazy_static;
extern crate image;
extern crate num_traits;
extern crate palette;
extern crate pbr;
extern crate pdqselect;
extern crate quickcheck;
extern crate rand;
extern crate rayon;
extern crate test;

use clap::{Arg, App};
use crossbeam_channel::{unbounded, Sender};
use euclid::*;
use palette::*;
use palette::pixel::Srgb;
use palette::white_point::E;
use pbr::ProgressBar;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::thread;

mod texture;
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
use hitable::triangle::*;
use material::*;
use random::*;
use types::*;
use texture::Texture;

fn color<T: CoordinateBase, H: Hitable<T>>(r: ray::Ray<T>, world: &H) -> Xyz<E, f32> {
    let refl = reflectance(r, world);
    color::xyz_from_wavelength(r.wl) * refl
}

fn reflectance<T: CoordinateBase, H: Hitable<T>>(r: ray::Ray<T>, world: &H) -> f32 {
    let mut r = r;
    let mut res = 0.0;
    let mut attenuation_acc = 1.0;
    for _ in 0..50 {
        let rec = world.hit(r, T::sqrt(T::epsilon()), T::max_value());
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
                let rgb = Rgb::with_wp(1.0, 1.0, 1.0)*(1.0-t) + Rgb::with_wp(0.5, 0.7, 1.0)*t;
                res += rgb.reflect(r.wl)*attenuation_acc;
                return res;
            }
        }
    }
    return res;
}

pub struct Scene<T> {
    objects: Vec<Arc<Hitable<f32>>>,
    look_from: Point3D<T>,
    look_at: Point3D<T>,
    focus_dist: T,
    aperture: T,
    vfov: T,
}

fn just_earth() -> Scene<f32> {
    let image = Arc::new(image::open("data/earth.jpg").unwrap().to_rgb());
    let texture: Arc<Texture<f32>> = Arc::new(texture::ImageTexture::new(&image));
    let material = Arc::new(Lambertian::new(&texture));
    let objects: Vec<Arc<Hitable<f32>>> = vec![
        Arc::new(Sphere::new(point3(0.0, 0.0, 0.0), 1.0, material)),
    ];

    let look_from = Point3D::new(3.0, -1.0, -1.5);
    let look_at = Point3D::new(0.0, 0.0, 0.0);
    let aperture = 0.0;
    let vfov = 35.0;
    let focus_dist = (look_from-look_at).length();

    Scene { objects, look_from, look_at, aperture, vfov, focus_dist }
}

fn three_spheres() -> Scene<f32> {
    let color: Arc<Texture<f32>> = Arc::new(Rgb::with_wp(0.1, 0.2, 0.5));
    let mat1 = Arc::new(Lambertian::new(&color));
    let color: Arc<Texture<f32>> = Arc::new(Rgb::with_wp(0.8, 0.8, 0.0));
    let mat2 = Arc::new(Lambertian::new(&color));
    let mat3 = Arc::new(Metal::new(Rgb::with_wp(0.8, 0.6, 0.2), 1.0));
    let mat4 = Arc::new(Dielectric::SF66);
    let objects: Vec<Arc<Hitable<f32>>> = vec![
        Arc::new(Sphere::new(Point3D::new(0.0, 0.0, -1.0), 0.5, mat1)),
        Arc::new(Sphere::new(Point3D::new(0.0, -100.5, -1.0), 100.0, mat2)),
        Arc::new(Sphere::new(Point3D::new(1.0, 0.0, -1.0), 0.5, mat3)),
        Arc::new(Sphere::new(Point3D::new(-1.0, 0.0, -1.0), 0.5, mat4.clone())),
        Arc::new(Sphere::new(Point3D::new(-1.25, 0.0, -1.0), -0.20, mat4.clone())),
        Arc::new(Sphere::new(Point3D::new(-0.75, 0.0, -1.0), -0.20, mat4)),
    ];

    let look_from = Point3D::new(-4.0, 0.7, 3.0);
    let look_at = Point3D::new(-1.0, 0.0, -1.0);
    let aperture = 0.1;
    let vfov = 15.0;
    let focus_dist = (look_from-look_at).length();

    Scene { objects, look_from, look_at, aperture, vfov, focus_dist }
}

fn many_spheres() -> Scene<f32> {
    let glass = Arc::new(Dielectric::SF66);
    let image = Arc::new(image::open("data/earth.jpg").unwrap().to_rgb());
    let texture: Arc<Texture<f32>> = Arc::new(texture::ImageTexture::new(&image));
    let ground = Arc::new(Lambertian::new(&texture));
    let color: Arc<Texture<f32>> = Arc::new(Rgb::with_wp(0.4, 0.2, 0.1));
    let sphere0_mat = Arc::new(Lambertian::new(&color));
    let sphere1_mat = Arc::new(Metal::new(Rgb::with_wp(0.7, 0.6, 0.5), 0.0));
    let mut objects: Vec<Arc<Hitable<f32>>> = vec![
        Arc::new(Triangle::new(
            (point3(-20.0, 0.0, -30.0), point3(-20.0, 0.0, 30.0), point3(20.0, 0.0, 30.0)),
            (vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0)),
            (vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0)),
            ground.clone(),
        )),
        Arc::new(Triangle::new(
            (point3(-20.0, 0.0, -30.0), point3(20.0, 0.0, -30.0), point3(20.0, 0.0, 30.0)),
            (vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0)),
            (vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0)),
            ground,
        )),
        Arc::new(Sphere::new(point3(0.0, 1.0, 0.0), 1.0, glass.clone())),
        Arc::new(Sphere::new(point3(-4.0, 1.0, 0.0), 1.0, sphere0_mat)),
        Arc::new(Sphere::new(point3(4.0, 1.0, 0.0), 1.0, sphere1_mat)),
    ];

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat: f32 = rand();
            let center = point3(a as f32+0.9*next_f32(), 0.2, b as f32+0.9*next_f32());
            if (center - vec3(4.0, 0.2, 0.0)).to_vector().length() > 0.9 {
                if choose_mat < 0.7 { // difuse
                    let color: Arc<Texture<f32>> =
                        Arc::new(Rgb::with_wp(
                            next_f32()*next_f32(),
                            next_f32()*next_f32(),
                            next_f32()*next_f32(),
                        ));
                    let mat = Arc::new(Lambertian::new(&color));
                    objects.push(Arc::new(Sphere::new(center, 0.2, mat)));
                } else if choose_mat < 0.85 { //metal
                    let color = Rgb::with_wp(
                        0.5*(1.0+next_f32()),
                        0.5*(1.0+next_f32()),
                        0.5*(1.0+next_f32()),
                    );
                    let mat = Arc::new(Metal::new(color, 0.5*next_f32()));
                    objects.push(Arc::new(Sphere::new(center, 0.2, mat)));
                } else {
                    objects.push(Arc::new(Sphere::new(center, 0.2, glass.clone())));
                }
            }
        }
    }

    let look_from = Point3D::new(13.0, 7.0, 3.0);
    let look_at = Point3D::new(0.0, -0.5, 0.0);
    let aperture = 0.0;
    let vfov = 30.0;
    let focus_dist = 10.0;

    Scene { objects, look_from, look_at, aperture, vfov, focus_dist }
}

lazy_static! {
    static ref SCENES: HashMap<&'static str, fn() -> Scene<f32>> = {
        let mut scenes: HashMap<_, fn() -> Scene<f32>> = HashMap::new();
        scenes.insert("just_earth", just_earth);
        scenes.insert("three_spheres", three_spheres);
        scenes.insert("many_spheres", many_spheres);
        scenes
    };
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
        .arg(Arg::with_name("cpuprofile")
             .long("cpuprofile")
             .value_name("FILE")
             .takes_value(true))
        .arg(Arg::with_name("scene")
             .long("scene")
             .value_name("SCENE_NAME")
             .default_value("many_spheres")
             .takes_value(true))
        .get_matches();

    let do_profile = match matches.value_of("output") {
        Some(out_file) => {
            cpuprofiler::PROFILER.lock().unwrap().start(out_file).unwrap();
            true
        },
        None => false
    };

    let output = Path::new(matches.value_of("output").unwrap());
    let format = match output.extension().map(|ext| ext.to_str().unwrap()) {
        None => panic!("Cannot know format without extension"),
        Some("png") => image::PNG,
        Some("jpg") => image::JPEG,
        Some("jpeg") => image::JPEG,
        Some(ext) => panic!("Unknown extension: {:?}", ext),
    };
    let output_str = String::from(output.to_str().unwrap());

    let get_scene: fn() -> Scene<f32> = match matches.value_of("scene").unwrap() {
        scene_name => match SCENES.get(scene_name) {
            Some(&get_scene) => get_scene,
            None => {
                panic!("Invalid scene {:?}, available: {:?}", scene_name, SCENES.keys());
            }
        }
    };

    let width: u32 = 800;
    let height: u32 = 600;
    let num_samples = 100;

    let Scene{ mut objects, look_from, look_at, aperture, vfov, focus_dist } = get_scene();
    let world = BVH::initialize(objects.as_mut_slice());
    let up = Vector3D::new(0.0, 1.0, 0.0);

    let cam = camera::Camera::new(look_from, look_at, up, vfov, width as f32/height as f32, aperture, focus_dist);

    let wl_low = 390.0;
    let wl_high = 700.0;
    let (sender, receiver): (Sender<Vec<_>>, _) = unbounded();
    let saver = thread::spawn(move|| {
        let mut pb = ProgressBar::new(num_samples);
        pb.format("╢▌▌░╟");
        let mut buffer = Vec::with_capacity((width*height) as usize);
        for _ in 0..width*height {
            buffer.push(Xyz::with_wp(0.0, 0.0, 0.0));
        };
        let mut samples_done = 0;
        while let Ok(sample) = receiver.recv() {
            let mut samples_pending = vec![sample];
            while let Ok(sample) = receiver.try_recv() {
                samples_pending.push(sample);
            }
            for i in 0..width*height {
                let mut acc = Xyz::with_wp(0.0, 0.0, 0.0);
                for sample in samples_pending.iter() {
                    acc = acc + sample[i as usize];
                };
                buffer[i as usize] = buffer[i as usize] + acc;
            };
            samples_done += samples_pending.len();

            let get_pixel = |x, y| {
                let col = buffer[(y*width+x) as usize];
                let col = (col.into_rgb()/(samples_done as f32)).clamp();
                let col = Srgb::from(col);
                let pixel =
                    [(col.red*255.99) as u8
                    ,(col.green*255.99) as u8
                    ,(col.blue*255.99) as u8
                    ];
                image::Rgb(pixel)
            };
            let buffer = image::ImageBuffer::from_fn(width, height, get_pixel);
            let output = Path::new(output_str.as_str());
            let ref mut fout = File::create(output).unwrap();
            image::ImageRgb8(buffer).save(fout, format).unwrap();
            pb.add(samples_pending.len() as u64);
        }
        pb.finish_print("done");
    });
    let _res: () =
        (0..num_samples)
        .into_par_iter()
        .map(|_| {
            let sample: Vec<Xyz<E, f32>> =
                (0..height*width)
                .into_par_iter()
                .map(|n| {
                    let i = n%width;
                    let j = height-(n/width);
                    let wl = gen_range(wl_low, wl_high);
                    let u = ((i as f32) + next_f32()) / (width as f32);
                    let v = ((j as f32) + next_f32()) / (height as f32);
                    let r = cam.get_ray(u, v, wl);
                    color(r, &world)*3.0
                }).collect();
            sender.send(sample).unwrap();
        }).collect();

    drop(sender);

    saver.join().unwrap();
    if do_profile {
        cpuprofiler::PROFILER.lock().unwrap().stop().unwrap();
    }
}
