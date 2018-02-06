use num_traits::Float;
use ray::Ray;
use euclid::*;

pub struct Camera<T> {
    lower_left_corner: Vector3D<T>,
    horizontal: Vector3D<T>,
    vertical: Vector3D<T>,
    origin: Point3D<T>,
}

impl Default for Camera<f32> {
    fn default() -> Camera<f32> {
        Camera {
            lower_left_corner: Vector3D::new(-2.0, -1.0, -1.0),
            horizontal: Vector3D::new(4.0, 0.0, 0.0),
            vertical: Vector3D::new(0.0, 2.0, 0.0),
            origin: Point3D::new(0.0, 0.0, 0.0)
        }
    }
}

impl<T: Float> Camera<T> {
    pub fn get_ray(&self, u: T, v: T) -> Ray<T> {
        Ray::new(self.origin, self.lower_left_corner + self.horizontal*u + self.vertical*v)
    }
}
