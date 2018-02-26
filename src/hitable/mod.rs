pub mod sphere;
pub mod hitable_list;
pub mod triangle;
pub mod bvh;

use num_traits::Float;
use euclid::*;

use ray::Ray;
use material::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct HitRecord<'a> {
    pub t: f32,
    pub p: Point3D<f32>,
    pub uv: Vector2D<f32>,
    pub normal: Vector3D<f32>,
    pub material: &'a Material,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct AABB {
    bounds: [Point3D<f32>;2]
}

impl AABB {
    pub fn intersects(&self, r: Ray, t0: f32, t1: f32) -> bool {
        match self {
            &AABB { bounds } => {
                let mut tmin = (bounds[r.sign.x as usize].x - r.origin.x) * r.inv_direction.x;
                let mut tmax = (bounds[1-r.sign.x as usize].x - r.origin.x) * r.inv_direction.x;
                let tymin = (bounds[r.sign.y as usize].y - r.origin.y) * r.inv_direction.y;
                let tymax = (bounds[1-r.sign.y as usize].y - r.origin.y) * r.inv_direction.y;
                if (tmin > tymax) || (tmax < tymin) {
                    return false;
                }
                if tymin>tmin {
                    tmin = tymin;
                }
                if tymax<tmax {
                    tmax = tymax;
                }
                let tzmin = (bounds[r.sign.z as usize].z - r.origin.z) * r.inv_direction.z;
                let tzmax = (bounds[1-r.sign.z as usize].z - r.origin.z) * r.inv_direction.z;
                if (tmin > tzmax) || (tmax < tzmin) {
                    return false;
                }
                if tzmin>tmin {
                    tmin = tzmin;
                }
                if tzmax<tmax {
                    tmax = tzmax;
                }
                (tmin < t1) && (tmax > t0)
            }
        }
    }

    pub fn empty() -> AABB {
        AABB {
            bounds: [
                point3(f32::max_value(), f32::max_value(), f32::max_value()),
                point3(f32::min_value(), f32::min_value(), f32::min_value()),
            ]
        }
    }

    pub fn merge(self, other: AABB) -> AABB {
        match (self, other) {
            (AABB { bounds: [low_0, high_0] }, AABB { bounds: [low_1, high_1] }) => {
                let low = point3(
                    f32::min(low_0.x, low_1.x),
                    f32::min(low_0.y, low_1.y),
                    f32::min(low_0.z, low_1.z),
                );
                let high = point3(
                    f32::max(high_0.x, high_1.x),
                    f32::max(high_0.y, high_1.y),
                    f32::max(high_0.z, high_1.z),
                );
                AABB { bounds: [low, high] }
            }
        }
    }
}

pub trait Hitable: Send + Sync {
    fn centroid(&self) -> Point3D<f32> {
        let bbox = self.bbox();
        match bbox {
            AABB{ bounds: [low, high] } => {
                point3(
                    (low.x + high.x)*0.5,
                    (low.y + high.y)*0.5,
                    (low.z + high.z)*0.5
                )
            }
        }
    }
    fn bbox(&self) -> AABB;
    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;
    use num_traits::Float;

    #[bench]
    fn bench_intersect_aabb_hit(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0));
        let aabb = black_box(AABB { bounds: [point3(-1.0, -1.0, -1.0), point3(1.0, 1.0, 1.0)] });
        bench.iter(|| aabb.intersects(ray, f32::epsilon(), f32::max_value()));
    }

    #[bench]
    fn bench_intersect_aabb_miss(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0));
        let aabb = black_box(AABB::empty());
        bench.iter(|| aabb.intersects(ray, f32::epsilon(), f32::max_value()));
    }
}
