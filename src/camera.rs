use num_traits::Float;
use ray::Ray;
use euclid::*;
use types::*;

pub struct Camera<T> {
    lower_left_corner: Vector3D<T>,
    horizontal: Vector3D<T>,
    vertical: Vector3D<T>,
    origin: Point3D<T>,
}

impl<T: CoordinateBase> Camera<T> {
    pub fn new(look_from: Point3D<T>, look_at: Point3D<T>, up: Vector3D<T>, vfov: T, aspect: T) -> Self {
        let theta = vfov*T::PI()/From::from(180.0);
        let half_height = T::tan(theta/From::from(2.0));
        let half_width = aspect * half_height;
        let origin = look_from;
        let w = (look_from - look_at).normalize();
        let u = up.cross(w).normalize();
        let v = w.cross(u);
        let lower_left_corner = -u*half_width - v*half_height - w;
        let horizontal = u*From::from(2.0)*half_width;
        let vertical = v*From::from(2.0)*half_height;
        Camera {
            lower_left_corner,
            horizontal,
            vertical,
            origin,
        }
    }
}

impl<T: Float> Camera<T> {
    pub fn get_ray(&self, u: T, v: T, wl: f32) -> Ray<T> {
        Ray::new(self.origin, self.lower_left_corner + self.horizontal*u + self.vertical*v, wl)
    }
}
