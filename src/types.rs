use num_traits::{Float};
use rand::distributions::range::SampleRange;
use std::fmt::Debug;

pub trait CoordinateBase: Float + SampleRange + Debug {
    fn from_f32(v: f32) -> Self;
}
impl CoordinateBase for f32 {
    #[inline]
    fn from_f32(v: f32) -> Self { v }
}
impl CoordinateBase for f64 {
    #[inline]
    fn from_f32(v: f32) -> Self { v as f64 }
}
