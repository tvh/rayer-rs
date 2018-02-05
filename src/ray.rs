use euclid::*;
use num_traits::float::*;

pub struct Ray<T> {
    pub origin: Point3D<T>,
    pub direction: Vector3D<T>
}

impl<T> Ray<T>
where
    T: Float
{
    pub fn new(origin: Point3D<T>, direction: Vector3D<T>) -> Ray<T> {
        Ray{origin, direction}
    }

    pub fn point_at_parameter(self, t: T) -> Point3D<T> {
        self.origin + self.direction*t
    }
}
