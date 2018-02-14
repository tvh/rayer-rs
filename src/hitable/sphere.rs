use euclid::*;
use ray::Ray;
use types::*;
use hitable::*;
use std::sync::Arc;
use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct Sphere<T: CoordinateBase> {
    center: Point3D<T>,
    radius: T,
    material: Arc<Material<T>>,
}

// This should not be necessary.
impl<T: CoordinateBase> PartialEq for Sphere<T> {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center &&
            self.radius == other.radius &&
            &self.material == &other.material
    }
}

impl<T: CoordinateBase> Sphere<T> {
    pub fn new(center: Point3D<T>, radius: T, material: Arc<Material<T>>) -> Sphere<T> {
        Sphere{
            center,
            radius,
            material
        }
    }
}

impl<T: CoordinateBase> Hitable<T> for Sphere<T> {
    fn centroid(&self) -> Point3D<T> {
        self.center
    }
    fn bbox(&self) -> AABB<T> {
        let abs_radius = self.radius.abs();
        let diff = vec3(abs_radius, abs_radius, abs_radius);
        AABB {
            low: self.center-diff,
            high: self.center+diff,
        }
    }
    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>> {
        let oc = r.origin - self.center;
        let a = r.direction.dot(r.direction);
        let b = oc.dot(r.direction);
        let c = oc.dot(oc) - self.radius*self.radius;
        let discriminant = b*b - a*c;
        if discriminant > T::zero() {
            let mut t = (-b - T::sqrt(discriminant))/a;
            if !(t < t_max && t > t_min) {
                t = (-b + T::sqrt(discriminant))/a;
            }
            if t < t_max && t > t_min {
                let p = r.point_at_parameter(t);
                let normal = (p-self.center) / self.radius;
                let phi = T::atan2(normal.z, normal.x);
                let theta = T::asin(normal.y);
                let u = T::one() - (phi+T::PI()) / (T::PI()+T::PI());
                let v = (theta + T::PI()*From::from(0.5)) / T::PI();
                let uv = vec2(u, v);
                return Some(HitRecord{normal, p, t, uv, material: self.material.borrow()});
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use palette::*;
    use texture::*;

    #[test]
    fn test_hit() {
        let color: Arc<Texture<f32>> = Arc::new(Rgb::new(0.5, 0.5, 0.5));
        let material: Arc<Material<f32>> = Arc::new(Lambertian::new(&color));
        let sphere = Sphere::new(point3(0.0, 0.0, 0.0), 1.0, material.clone());
        let ray = Ray::new(point3(-2.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0);
        let res = sphere.hit(ray, 0.0, 1000.0);
        match res {
            None => panic!("Expected a hit"),
            Some(hit) => {
                let t = 1.0;
                let p = point3(-1.0, 0.0, 0.0);
                let normal = vec3(-1.0, 0.0, 0.0);
                let uv = vec2(0.0, 0.5);
                let expected = HitRecord{t, p, normal, uv, material: material.borrow()};
                assert_eq!(expected, hit);
            }
        }
        let ray = Ray::new(point3(1.5, 0.0, 0.0), vec3(-1.0, 0.0, 0.0), 500.0);
        let res = sphere.hit(ray, 0.0, 1000.0);
        match res {
            None => panic!("Expected a hit"),
            Some(hit) => {
                let t = 0.5;
                let p = point3(1.0, 0.0, 0.0);
                let normal = vec3(1.0, 0.0, 0.0);
                let uv = vec2(0.5, 0.5);
                let expected = HitRecord{t, p, normal, uv, material: material.borrow()};
                assert_eq!(expected, hit);
            }
        }
        let ray = Ray::new(point3(0.0, 3.0, 0.0), vec3(0.0, -1.0, 0.0), 500.0);
        let res = sphere.hit(ray, 0.0, 1000.0);
        match res {
            None => panic!("Expected a hit"),
            Some(hit) => {
                let t = 2.0;
                let p = point3(0.0, 1.0, 0.0);
                let normal = vec3(0.0, 1.0, 0.0);
                let uv = vec2(0.5, 1.0);
                let expected = HitRecord{t, p, normal, uv, material: material.borrow()};
                assert_eq!(expected, hit);
            }
        }
    }
}
