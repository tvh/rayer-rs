use num_traits::{Float};
use rand::distributions::range::SampleRange;
use std::fmt::Debug;
use rand::Rand;

pub trait CoordinateBase: Float + SampleRange + Debug + From<f32> + Rand {}
impl<T: Float + SampleRange + Debug + From<f32> + Rand> CoordinateBase for T {}

