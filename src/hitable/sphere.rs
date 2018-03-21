use euclid::*;
use ray::Ray;
use hitable::*;
use std::sync::Arc;
use num_traits::FloatConst;
use texture::Texture;

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3D<f32>,
    radius: f32,
    texture: Arc<Texture>,
}

// This should not be necessary.
impl PartialEq for Sphere {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center &&
            self.radius == other.radius &&
            &self.texture == &other.texture
    }
}

impl Sphere {
    pub fn new(center: Point3D<f32>, radius: f32, texture: Arc<Texture>) -> Sphere {
        Sphere{
            center,
            radius,
            texture,
        }
    }
}

impl Hitable for Sphere {
    fn centroid(&self) -> Point3D<f32> {
        self.center
    }
    fn bbox(&self) -> AABB {
        let abs_radius = self.radius.abs();
        let diff = vec3(abs_radius, abs_radius, abs_radius);
        AABB {
            bounds: [self.center-diff, self.center+diff]
        }
    }
    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = r.direction.dot(r.direction);
        let b = oc.dot(r.direction);
        let c = oc.dot(oc) - self.radius*self.radius;
        let discriminant = b*b - a*c;
        if discriminant > 0.0 {
            let mut t = (-b - f32::sqrt(discriminant))/a;
            if !(t < t_max && t > t_min) {
                t = (-b + f32::sqrt(discriminant))/a;
            }
            if t < t_max && t > t_min {
                let p = r.point_at_parameter(t);
                let normal = (p-self.center) / self.radius;
                let phi = f32::atan2(normal.z, normal.x);
                let theta = f32::asin(normal.y);
                let u = 1.0 - (phi+f32::PI()) / (f32::PI()+f32::PI());
                let v = (theta + f32::PI()*0.5) / f32::PI();
                let uv = vec2(u, v);
                return Some(HitRecord{normal, p, t, uv, texture: self.texture.as_ref()});
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use palette::*;
    use material::*;

    #[test]
    fn test_hit() {
        let texture: Arc<Texture> = Arc::new(Lambertian::new(Rgb::with_wp(0.5, 0.5, 0.5)));
        let sphere = Sphere::new(point3(0.0, 0.0, 0.0), 1.0, texture.clone());
        let ray = Ray::new(point3(-2.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0, 0.0);
        let res = sphere.hit(ray, 0.0, 1000.0);
        match res {
            None => panic!("Expected a hit"),
            Some(hit) => {
                let t = 1.0;
                let p = point3(-1.0, 0.0, 0.0);
                let normal = vec3(-1.0, 0.0, 0.0);
                let uv = vec2(0.0, 0.5);
                let expected = HitRecord{t, p, normal, uv, texture: texture.as_ref()};
                assert_eq!(expected, hit);
            }
        }
        let ray = Ray::new(point3(1.5, 0.0, 0.0), vec3(-1.0, 0.0, 0.0), 500.0, 0.0);
        let res = sphere.hit(ray, 0.0, 1000.0);
        match res {
            None => panic!("Expected a hit"),
            Some(hit) => {
                let t = 0.5;
                let p = point3(1.0, 0.0, 0.0);
                let normal = vec3(1.0, 0.0, 0.0);
                let uv = vec2(0.5, 0.5);
                let expected = HitRecord{t, p, normal, uv, texture: texture.as_ref()};
                assert_eq!(expected, hit);
            }
        }
        let ray = Ray::new(point3(0.0, 3.0, 0.0), vec3(0.0, -1.0, 0.0), 500.0, 0.0);
        let res = sphere.hit(ray, 0.0, 1000.0);
        match res {
            None => panic!("Expected a hit"),
            Some(hit) => {
                let t = 2.0;
                let p = point3(0.0, 1.0, 0.0);
                let normal = vec3(0.0, 1.0, 0.0);
                let uv = vec2(0.5, 1.0);
                let expected = HitRecord{t, p, normal, uv, texture: texture.as_ref()};
                assert_eq!(expected, hit);
            }
        }
    }
}
