use ray::Ray;
use euclid::*;
use types::*;
use random::*;

pub struct Camera<T> {
    origin: Point3D<T>,
    lower_left_corner: Vector3D<T>,
    horizontal: Vector3D<T>,
    vertical: Vector3D<T>,
    u: Vector3D<T>,
    v: Vector3D<T>,
    lens_radius: T,
}

impl<T: CoordinateBase> Camera<T> {
    pub fn new(look_from: Point3D<T>, look_at: Point3D<T>, up: Vector3D<T>, vfov: T, aspect: T, aperture: T, focus_dist: T) -> Self {
        let lens_radius = aperture/From::from(2.0);
        let theta = vfov.to_radians();
        let half_height = T::tan(theta/From::from(2.0));
        let half_width = aspect * half_height;
        let origin = look_from;
        let w = (look_from - look_at).normalize();
        let u = up.cross(w).normalize();
        let v = w.cross(u);
        let lower_left_corner = -u*half_width*focus_dist - v*half_height*focus_dist - w*focus_dist;
        let horizontal = u*From::from(2.0)*half_width*focus_dist;
        let vertical = v*From::from(2.0)*half_height*focus_dist;
        Camera {
            lower_left_corner,
            horizontal,
            vertical,
            origin,
            u, v,
            lens_radius,
        }
    }
}

impl<T: CoordinateBase> Camera<T> {
    pub fn get_ray(&self, s: T, t: T, wl: f32) -> Ray<T> {
        let rd = rand_in_unit_disk()*self.lens_radius;
        let offset = self.u*rd.x + self.v*rd.y;
        Ray::new(self.origin + offset, self.lower_left_corner + self.horizontal*s + self.vertical*t - offset, wl)
    }
}
