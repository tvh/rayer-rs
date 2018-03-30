#![feature(const_fn)]
#![feature(fixed_size_array)]
#![feature(plugin)]
#![feature(slice_patterns)]
#![feature(test)]
#![feature(underscore_lifetimes)]
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
extern crate obj;
extern crate palette;
extern crate pbr;
extern crate pdqselect;
extern crate quickcheck;
extern crate rand;
extern crate rayon;
extern crate tempfile;
extern crate test;

use clap::{Arg, App};
use crossbeam_channel::{unbounded, Sender};
use euclid::*;
use image::hdr::*;
use num_traits::Float;
use palette::*;
use palette::pixel::Srgb;
use palette::white_point::E;
use pbr::ProgressBar;
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;

mod texture;
mod camera;
mod color;
mod hitable;
mod material;
mod random;
mod ray;

use color::HasReflectance;
use hitable::Hitable;
use hitable::bvh::*;
use hitable::sphere::*;
use hitable::triangle::*;
use material::*;
use random::*;
use texture::Texture;

fn color<H: Hitable>(r: ray::Ray, world: &H, render_sky: bool) -> Xyz<E, f32> {
    let refl = reflectance(r, world, render_sky);
    color::xyz_from_wavelength(r.wl) * refl
}

fn reflectance<H: Hitable>(r: ray::Ray, world: &H, render_sky: bool) -> f32 {
    let mut r = r;
    let mut res = 0.0;
    let mut attenuation_acc = 1.0;
    for _ in 0..50 {
        let rec = world.hit(r, f32::sqrt(f32::epsilon()), f32::max_value());
        match rec {
            Some(rec) => {
                let mat = rec.texture.value(rec.uv);
                let mat_res = mat.scatter(r, rec);
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
                if render_sky {
                    let unit_direction = r.direction.normalize();
                    let t: f32 = (unit_direction.y + 1.0)*0.5;
                    let rgb = Rgb::with_wp(1.0, 1.0, 1.0)*(1.0-t) + Rgb::with_wp(0.5, 0.7, 1.0)*t;
                    res += rgb.reflect(r.wl)*attenuation_acc;
                }
                return res;
            }
        }
    }
    return res;
}

pub struct Scene {
    objects: Vec<Arc<Hitable>>,
    look_from: Point3D<f32>,
    look_at: Point3D<f32>,
    focus_dist: f32,
    aperture: f32,
    vfov: f32,
    render_sky: bool,
}

fn just_earth() -> Scene {
    let image = Arc::new(image::open("data/earth.jpg").unwrap().to_rgb());
    let texture: Arc<Texture> = Arc::new(texture::ImageTexture::new(&image));
    let objects: Vec<Arc<Hitable>> = vec![
        Arc::new(Sphere::new(point3(0.0, 0.0, 0.0), 1.0, texture)),
    ];

    let look_from = Point3D::new(3.0, -1.0, -1.5);
    let look_at = Point3D::new(0.0, 0.0, 0.0);
    let aperture = 0.0;
    let vfov = 35.0;
    let focus_dist = (look_from-look_at).length();
    let render_sky = true;

    Scene { objects, look_from, look_at, aperture, vfov, focus_dist, render_sky }
}

fn three_spheres() -> Scene {
    let mat1 = Arc::new(Lambertian::new(Rgb::with_wp(0.1, 0.2, 0.5)));
    let mat2 = Arc::new(Lambertian::new(Rgb::with_wp(0.8, 0.8, 0.0)));
    let mat3 = Arc::new(Metal::new(Rgb::with_wp(0.8, 0.6, 0.2), 1.0));
    let mat4 = Arc::new(Dielectric::SF66);
    let objects: Vec<Arc<Hitable>> = vec![
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
    let render_sky = true;

    Scene { objects, look_from, look_at, aperture, vfov, focus_dist, render_sky }
}

fn many_spheres() -> Scene {
    let glass = Arc::new(Dielectric::SF66);
    let image = Arc::new(image::open("data/earth.jpg").unwrap().to_rgb());
    let ground: Arc<Texture> = Arc::new(texture::ImageTexture::new(&image));
    let sphere0_mat = Arc::new(Lambertian::new(Rgb::with_wp(0.4, 0.2, 0.1)));
    let sphere1_mat = Arc::new(Metal::new(Rgb::with_wp(0.7, 0.6, 0.5), 0.0));
    let mut objects: Vec<Arc<Hitable>> = vec![
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
                if choose_mat < 0.8 { // difuse
                    let color =
                        Rgb::with_wp(
                            next_f32()*next_f32(),
                            next_f32()*next_f32(),
                            next_f32()*next_f32(),
                        );
                    let mat = Arc::new(Lambertian::new(color));
                    let center1 = center + vec3(0.0, 0.5*next_f32(), 0.0);
                    objects.push(Arc::new(Sphere::new_moving(center, center1, 0.0, 1.0, 0.2, mat)));
                } else if choose_mat < 0.95 { //metal
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

    let look_from = Point3D::new(13.0, 2.0, 3.0);
    let look_at = Point3D::new(0.0, 0.0, 0.0);
    let aperture = 0.1;
    let vfov = 30.0;
    let focus_dist = 10.0;
    let render_sky = true;

    Scene { objects, look_from, look_at, aperture, vfov, focus_dist, render_sky }
}

fn simple_light() -> Scene {
    let glass = Arc::new(Dielectric::SF66);
    let ground = Arc::new(Lambertian::new(Rgb::with_wp(0.5, 0.5, 0.5)));
    let light = Arc::new(light::DiffuseLight::new(Rgb::with_wp(5.0, 5.0, 5.0)));
    let image = Arc::new(image::open("data/earth.jpg").unwrap().to_rgb());
    let sphere0_mat: Arc<Texture> = Arc::new(texture::ImageTexture::new(&image));
    let sphere1_mat = Arc::new(Metal::new(Rgb::with_wp(0.7, 0.6, 0.5), 0.0));
    let objects: Vec<Arc<Hitable>> = vec![
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
        Arc::new(Sphere::new(point3(0.0, 1.3, 0.0), -0.70, glass.clone())),
        Arc::new(Sphere::new(point3(-3.0, 1.0, 0.0), 1.0, sphere0_mat)),
        Arc::new(Sphere::new(point3(3.0, 1.0, 0.0), 1.0, sphere1_mat)),
        Arc::new(Sphere::new(point3(0.0, 6.0, 2.0), 2.0, light.clone())),
    ];

    let look_from = Point3D::new(0.0, 2.0, -10.0);
    let look_at = Point3D::new(0.0, 1.0, 0.0);
    let aperture = 0.1;
    let vfov = 30.0;
    let focus_dist = 10.0;
    let render_sky = false;

    Scene { objects, look_from, look_at, aperture, vfov, focus_dist, render_sky }
}

fn bunny() -> Scene {
    let light = Arc::new(light::DiffuseLight::new(Rgb::with_wp(5.0, 5.0, 5.0)));
    let ground = Arc::new(Lambertian::new(Rgb::with_wp(0.5, 0.5, 0.5)));
    let bunny0_mat = Arc::new(Dielectric::SF66);
    let bunny0 = Mesh::from_obj(Path::new("data/bunny.obj"), bunny0_mat).unwrap();
    let objects: Vec<Arc<Hitable>> = vec![
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
        Arc::new(bunny0),
        Arc::new(Sphere::new(point3(0.0, 6.0, -2.0), 2.0, light.clone())),
    ];

    let look_from = Point3D::new(0.0, 2.0, 10.0);
    let look_at = Point3D::new(0.0, 1.0, 0.0);
    let aperture = 0.1;
    let vfov = 30.0;
    let focus_dist = 10.0;
    let render_sky = false;

    Scene { objects, look_from, look_at, aperture, vfov, focus_dist, render_sky }
}

fn cornell() -> Scene {
    let red = Arc::new(Lambertian::new(Rgb::with_wp(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Rgb::with_wp(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Rgb::with_wp(0.12, 0.45, 0.15)));
    let light = Arc::new(light::DiffuseLight::new(Rgb::with_wp(15.0, 15.0, 15.0)));
    let up = vec3(0.0, 1.0, 0.0);
    let down = vec3(0.0, -1.0, 0.0);
    let right = vec3(-1.0, 0.0, 0.0);
    let left = vec3(1.0, 0.0, 0.0);
    let out = vec3(0.0, 0.0, -1.0);
    let mut triangles: Vec<Triangle> = Vec::new();
    triangles.extend(uniform_polygon(
        &[point3(213.0, 554.0, 227.0), point3(213.0, 554.0, 332.0),
          point3(343.0, 554.0, 332.0), point3(343.0, 554.0, 227.0)],
        down,
        light
    ));
    triangles.extend(uniform_polygon(
        &[point3(0.0, 555.0, 0.0), point3(0.0, 555.0, 555.0),
          point3(555.0, 555.0, 555.0), point3(555.0, 555.0, 0.0)],
        down,
        white.clone()
    ));
    triangles.extend(uniform_polygon(
        &[point3(0.0, 0.0, 0.0), point3(0.0, 0.0, 555.0),
          point3(555.0, 0.0, 555.0), point3(555.0, 0.0, 0.0)],
        up,
        white.clone()
    ));
    triangles.extend(uniform_polygon(
        &[point3(0.0, 0.0, 555.0), point3(0.0, 555.0, 555.0),
          point3(555.0, 555.0, 555.0), point3(555.0, 0.0, 555.0)],
        out,
        white.clone()
    ));
    triangles.extend(uniform_polygon(
        &[point3(0.0, 0.0, 0.0), point3(0.0, 0.0, 555.0),
          point3(0.0, 555.0, 555.0), point3(0.0, 555.0, 0.0)],
        left,
        red
    ));
    triangles.extend(uniform_polygon(
        &[point3(555.0, 0.0, 0.0), point3(555.0, 0.0, 555.0),
          point3(555.0, 555.0, 555.0), point3(555.0, 555.0, 0.0)],
        right,
        green
    ));
    let mut objects: Vec<Arc<Hitable>> =
        triangles
        .iter()
        .map(|t| Arc::new(t.clone()) as Arc<Hitable>)
        .collect();

    objects.push(Arc::new(axis_aligned_cuboid(
        point3(130.0, 0.0, 65.0),
        point3(295.0, 165.0, 230.0),
        white.clone()
    )));
    objects.push(Arc::new(axis_aligned_cuboid(
        point3(265.0, 0.0, 295.0),
        point3(430.0, 330.0, 460.0),
        white.clone()
    )));

    let look_from = Point3D::new(278.0, 278.0, -800.0);
    let look_at = Point3D::new(278.0, 278.0, 0.0);
    let aperture = 0.0;
    let vfov = 40.0;
    let focus_dist = 10.0;
    let render_sky = false;

    Scene { objects, look_from, look_at, aperture, vfov, focus_dist, render_sky }
}

lazy_static! {
    static ref SCENES: HashMap<&'static str, fn() -> Scene> = {
        let mut scenes: HashMap<_, fn() -> Scene> = HashMap::new();
        scenes.insert("just_earth", just_earth);
        scenes.insert("three_spheres", three_spheres);
        scenes.insert("many_spheres", many_spheres);
        scenes.insert("simple_light", simple_light);
        scenes.insert("bunny", bunny);
        scenes.insert("cornell", cornell);
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
        .arg(Arg::with_name("samples")
             .long("samples")
             .value_name("NUMBER")
             .default_value("100")
             .takes_value(true))
        .arg(Arg::with_name("width")
             .long("width")
             .value_name("NUMBER")
             .takes_value(true))
        .arg(Arg::with_name("height")
             .long("height")
             .value_name("NUMBER")
             .takes_value(true))
        .get_matches();

    let do_profile = match matches.value_of("cpuprofile") {
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
        Some("hdr") => image::ImageFormat::HDR,
        Some(ext) => panic!("Unknown extension: {:?}", ext),
    };
    let output_str = String::from(output.to_str().unwrap());

    let get_scene: fn() -> Scene = match matches.value_of("scene").unwrap() {
        scene_name => match SCENES.get(scene_name) {
            Some(&get_scene) => get_scene,
            None => {
                panic!("Invalid scene {:?}, available: {:?}", scene_name, SCENES.keys());
            }
        }
    };

    let def_width: u32 = 800;
    let def_height: u32 = 600;
    let (width, height) = match (matches.value_of("width"), matches.value_of("height")) {
        (None, None) => (def_width, def_height),
        (Some(width_str), None) => {
            let width = u32::from_str(width_str).unwrap();
            let height = width*def_height/def_width;
            (width, height)
        },
        (None, Some(height_str)) => {
            let height = u32::from_str(height_str).unwrap();
            let width = height*def_width/def_height;
            (width, height)
        },
        (Some(width_str), Some(height_str)) => {
            let width = u32::from_str(width_str).unwrap();
            let height = u32::from_str(height_str).unwrap();
            (width, height)
        },
    };
    let num_samples = u64::from_str(matches.value_of("samples").unwrap()).unwrap();

    let Scene{ mut objects, look_from, look_at, aperture, vfov, focus_dist, render_sky } = get_scene();
    let world = BVH::initialize(objects.as_mut_slice());
    let up = Vector3D::new(0.0, 1.0, 0.0);

    let cam = camera::Camera::new(look_from, look_at, up, vfov, width as f32/height as f32, aperture, focus_dist, 0.0, 1.0);

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
        let output_path = Path::new(output_str.as_str());
        let output_dir = output_path.parent().unwrap();
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
                col.into_rgb()/(samples_done as f32)
            };
            let get_pixel_hdr = |x, y| {
                let col = get_pixel(x, y);
                image::Rgb([col.red, col.green, col.blue])
            };
            let get_pixel_ldr = |x, y| {
                let col = get_pixel(x, y);
                let col = Srgb::from(col.clamp());
                let pixel =
                    [(col.red*255.99) as u8
                    ,(col.green*255.99) as u8
                    ,(col.blue*255.99) as u8
                    ];
                image::Rgb(pixel)
            };

            let mut fout = tempfile::NamedTempFile::new_in(output_dir).unwrap();

            match format {
                image::ImageFormat::HDR => {
                    let buffer: Vec<_> =
                        (0..(width*height))
                        .map(|n| get_pixel_hdr(n%width, n/width))
                        .collect();
                    let encoder = HDREncoder::new(&fout);
                    encoder.encode(buffer.as_slice(), width as usize, height as usize).unwrap();
                },
                _ => {
                    let buffer = image::ImageBuffer::from_fn(width, height, get_pixel_ldr);
                    image::ImageRgb8(buffer).save(&mut fout, format).unwrap();
                }
            }
            fout.flush().unwrap();
            fout.persist(&output_path).unwrap();
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
                    color(r, &world, render_sky)*3.0
                }).collect();
            sender.send(sample).unwrap();
        }).collect();

    drop(sender);

    saver.join().unwrap();
    if do_profile {
        cpuprofiler::PROFILER.lock().unwrap().stop().unwrap();
    }
}
