use num_traits::{Float, FloatConst};
use rand::distributions::range::SampleRange;
use std::fmt::Debug;
use rand::Rand;
use decorum::*;

pub trait CoordinateBase: Float + SampleRange + Debug + From<f32> + Rand + FloatConst + Send + Sync + Primitive {}
impl<T: Float + SampleRange + Debug + From<f32> + Rand + FloatConst + Send + Sync + Primitive> CoordinateBase for T {}

