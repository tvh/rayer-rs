use ray::Ray;
use euclid::*;
use random::*;

pub struct Camera {
    origin: Point3D<f32>,
    lower_left_corner: Vector3D<f32>,
    horizontal: Vector3D<f32>,
    vertical: Vector3D<f32>,
    u: Vector3D<f32>,
    v: Vector3D<f32>,
    lens_radius: f32,
}

impl Camera {
    pub fn new(look_from: Point3D<f32>, look_at: Point3D<f32>, up: Vector3D<f32>, vfov: f32, aspect: f32, aperture: f32, focus_dist: f32) -> Self {
        let lens_radius = aperture*0.5;
        let theta = vfov.to_radians();
        let half_height = f32::tan(theta*0.5);
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

impl Camera {
    pub fn get_ray(&self, s: f32, t: f32, wl: f32) -> Ray {
        let rd = rand_in_unit_disk()*self.lens_radius;
        let offset = self.u*rd.x + self.v*rd.y;
        Ray::new(self.origin + offset, self.lower_left_corner + self.horizontal*s + self.vertical*t - offset, wl, 0.0)
    }
}
