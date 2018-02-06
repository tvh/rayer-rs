use euclid::*;
use num_traits::float::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Ray<T> {
    pub origin: Point3D<T>,
    pub direction: Vector3D<T>,
    pub wl: f32,
}

impl<T> Ray<T>
where
    T: Float
{
    pub fn new(origin: Point3D<T>, direction: Vector3D<T>, wl: f32) -> Ray<T> {
        Ray{origin, direction, wl}
    }

    pub fn point_at_parameter(self, t: T) -> Point3D<T> {
        self.origin + self.direction*t
    }
}
