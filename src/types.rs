use num_traits::{Float, FloatConst};
use rand::distributions::range::SampleRange;
use std::fmt::Debug;
use rand::Rand;

pub trait CoordinateBase: Float + SampleRange + Debug + From<f32> + Rand + FloatConst + Send + Sync {}
impl<T: Float + SampleRange + Debug + From<f32> + Rand + FloatConst + Send + Sync> CoordinateBase for T {}

