pub mod sphere;
pub mod hitable_list;
pub mod triangle;
pub mod bvh;

use num_traits::Float;
use euclid::*;

use ray::Ray;
use texture::*;

#[derive(PartialEq, Debug)]
pub struct HitRecord<'a> {
    pub t: f32,
    pub p: Point3D<f32>,
    pub uv: Vector2D<f32>,
    pub normal: Vector3D<f32>,
    pub texture: &'a Texture,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct AABB {
    bounds: [Point3D<f32>;2]
}

impl AABB {
    pub fn intersects(&self, r: Ray, t0: f32, t1: f32) -> Option<(f32, f32)> {
        match self {
            &AABB { bounds } => {
                let mut tmin = (bounds[r.sign.x as usize].x - r.origin.x) * r.inv_direction.x;
                let mut tmax = (bounds[1-r.sign.x as usize].x - r.origin.x) * r.inv_direction.x;
                let tymin = (bounds[r.sign.y as usize].y - r.origin.y) * r.inv_direction.y;
                let tymax = (bounds[1-r.sign.y as usize].y - r.origin.y) * r.inv_direction.y;
                if (tmin > tymax) || (tmax < tymin) {
                    return None;
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
                    return None;
                }
                if tzmin>tmin {
                    tmin = tzmin;
                }
                if tzmax<tmax {
                    tmax = tzmax;
                }
                if (tmin < t1) && (tmax > t0) {
                    Some((tmin, tmax))
                } else {
                    None
                }
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
                    if low_0.x<low_1.x { low_0.x } else { low_1.x },
                    if low_0.y<low_1.y { low_0.y } else { low_1.y },
                    if low_0.z<low_1.z { low_0.z } else { low_1.z },
                );
                let high = point3(
                    if high_0.x>high_1.x { high_0.x } else { high_1.x },
                    if high_0.y>high_1.y { high_0.y } else { high_1.y },
                    if high_0.z>high_1.z { high_0.z } else { high_1.z },
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

    #[bench]
    fn bench_merge_aabb(bench: &mut Bencher) {
        let aabb1 = black_box(AABB { bounds: [point3(-1.0, -1.0, -1.0), point3(0.0, 0.0, 0.0)] });
        let aabb2 = black_box(AABB { bounds: [point3(0.0, 0.0, 0.0), point3(1.0, 1.0, 1.0)] });
        bench.iter(|| aabb1.merge(aabb2) );
    }
}
