#![feature(const_fn)]
#![feature(fixed_size_array)]
#![feature(plugin)]
#![feature(slice_patterns)]
#![feature(stdsimd)]
#![feature(test)]
#![feature(underscore_lifetimes)]
#![plugin(quickcheck_macros)]
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
extern crate quickcheck;
extern crate rand;
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
