pub mod sphere;
pub mod hitable_list;
pub mod triangle;
pub mod bvh;

use euclid::*;

use types::*;
use ray::Ray;
use material::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct HitRecord<'a, T: 'a + CoordinateBase> {
    pub t: T,
    pub p: Point3D<T>,
    pub uv: Vector2D<T>,
    pub normal: Vector3D<T>,
    pub material: &'a Material<T>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct AABB<T: CoordinateBase> {
    bounds: [Point3D<T>;2]
}

impl<T: CoordinateBase> AABB<T> {
    pub fn intersects(&self, r: Ray<T>, t0: T, t1: T) -> bool {
        match self {
            &AABB { bounds } => {
                let tmin = (bounds[r.sign.x as usize].x - r.origin.x) * r.inv_direction.x;
                let tmax = (bounds[1-r.sign.x as usize].x - r.origin.x) * r.inv_direction.x;
                let tymin = (bounds[r.sign.y as usize].y - r.origin.y) * r.inv_direction.y;
                let tymax = (bounds[1-r.sign.y as usize].y - r.origin.y) * r.inv_direction.y;
                if (tmin > tymax) || (tmax < tymin) {
                    return false;
                }
                let tmin = T::max(tmin, tymin);
                let tmax = T::min(tmax, tymax);
                let tzmin = (bounds[r.sign.z as usize].z - r.origin.z) * r.inv_direction.z;
                let tzmax = (bounds[1-r.sign.z as usize].z - r.origin.z) * r.inv_direction.z;
                if (tmin > tzmax) || (tmax < tzmin) {
                    return false;
                }
                let tmin = T::max(tmin, tzmin);
                let tmax = T::min(tmax, tzmax);
                (tmin < t1) && (tmax > t0)
            }
        }
    }

    pub fn empty() -> AABB<T> {
        AABB {
            bounds: [
                point3(T::max_value(), T::max_value(), T::max_value()),
                point3(T::min_value(), T::min_value(), T::min_value()),
            ]
        }
    }

    pub fn merge(self, other: AABB<T>) -> AABB<T> {
        match (self, other) {
            (AABB { bounds: [low_0, high_0] }, AABB { bounds: [low_1, high_1] }) => {
                let low = point3(
                    T::min(low_0.x, low_1.x),
                    T::min(low_0.y, low_1.y),
                    T::min(low_0.z, low_1.z),
                );
                let high = point3(
                    T::max(high_0.x, high_1.x),
                    T::max(high_0.y, high_1.y),
                    T::max(high_0.z, high_1.z),
                );
                AABB { bounds: [low, high] }
            }
        }
    }
}

pub trait Hitable<T: CoordinateBase> {
    fn centroid(&self) -> Point3D<T> {
        let bbox = self.bbox();
        match bbox {
            AABB{ bounds: [low, high] } => {
                point3(
                    (low.x + high.x)*From::from(0.5),
                    (low.y + high.y)*From::from(0.5),
                    (low.z + high.z)*From::from(0.5)
                )
            }
        }
    }
    fn bbox(&self) -> AABB<T>;
    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;
    use num_traits::Float;

    #[bench]
    fn bench_intersect_aabb_hit_f32(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0));
        let aabb = black_box(AABB { bounds: [point3(-1.0, -1.0, -1.0), point3(1.0, 1.0, 1.0)] });
        bench.iter(|| aabb.intersects(ray, f32::epsilon(), f32::max_value()));
    }

    #[bench]
    fn bench_intersect_aabb_miss_f32(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0));
        let aabb = black_box(AABB::empty());
        bench.iter(|| aabb.intersects(ray, f32::epsilon(), f32::max_value()));
    }

    #[bench]
    fn bench_intersect_aabb_hit_f64(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0));
        let aabb = black_box(AABB {bounds: [point3(-1.0, -1.0, -1.0), point3(1.0, 1.0, 1.0) ]});
        bench.iter(|| aabb.intersects(ray, f64::epsilon(), f64::max_value()));
    }

    #[bench]
    fn bench_intersect_aabb_miss_f64(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0));
        let aabb = black_box(AABB::empty());
        bench.iter(|| aabb.intersects(ray, f64::epsilon(), f64::max_value()));
    }
}
