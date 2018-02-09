use euclid::*;
use ray::Ray;
use types::*;
use hitable::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Sphere<T> {
    center: Point3D<T>,
    radius: T
}

impl<T: CoordinateBase> Sphere<T> {
    pub fn new(center: Point3D<T>, radius: T) -> Sphere<T> {
        Sphere{center, radius}
    }
}

impl<T: CoordinateBase> Hitable<T> for Sphere<T> {
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
                return Some(HitRecord{normal, p, t, material: &Lambertian});
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit() {
        // TODO: Do more than this smoke test
        let sphere = Sphere::new(Point3D::new(0.0, 0.0, 0.0), 1.0);
        let ray = Ray::new(Point3D::new(-3.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0), 500.0);
        let res = sphere.hit(ray, 0.0, 1000.0);
        match res {
            None => panic!("Expected a hit"),
            Some(hit) => {
                let t = 2.0;
                let p = Point3D::new(-1.0, 0.0, 0.0);
                let normal = Vector3D::new(-1.0, 0.0, 0.0);
                let expected = HitRecord{t, p, normal, material: &Lambertian};
                assert_eq!(expected, hit);
            }
        }
    }
}
