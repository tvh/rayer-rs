use num_traits::Float;
use rand::distributions::range::SampleRange;

pub trait CoordinateBase: Float + SampleRange {}
impl<T: Float + SampleRange> CoordinateBase for T {}
