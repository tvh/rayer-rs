#![feature(stdsimd)]
#![feature(test)]
extern crate arrayvec;
extern crate core;
extern crate clap;
extern crate cpuprofiler;
extern crate crossbeam_channel;
extern crate decorum;
extern crate euclid;
extern crate image;
extern crate num_traits;
extern crate obj;
extern crate palette;
extern crate pbr;
extern crate pdqselect;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate packed_simd;
extern crate rand;
extern crate rand_xorshift;
extern crate rand_xoshiro;
extern crate rayon;
extern crate tempfile;
extern crate test;

pub mod texture;
pub mod camera;
pub mod color;
pub mod hitable;
pub mod material;
pub mod random;
pub mod ray;
