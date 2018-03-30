pub mod sphere;
pub mod triangle;
pub mod bvh;
pub mod instance;

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
    pub bounds: [Point3D<f32>;2]
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

    pub fn intersects_2(&self, second: &Self, r: Ray, t0: f32, t1: f32) -> (Option<(f32, f32)>, Option<(f32, f32)>) {
        (self.intersects(r, t0, t1), second.intersects(r, t0, t1))
    }

    pub fn empty() -> AABB {
        AABB {
            bounds: [
                point3(f32::max_value(), f32::max_value(), f32::max_value()),
                point3(f32::min_value(), f32::min_value(), f32::min_value()),
            ]
        }
    }

    pub fn is_empty(self) -> bool {
        match self {
            AABB { bounds: [low, high] } => {
                low.x>high.x || low.y>high.y || low.z>high.z
            }
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

impl<T: AsRef<Hitable> + Sync + Send> Hitable for T {
    fn centroid(&self) -> Point3D<f32> {
         self.as_ref().centroid()
    }
    fn bbox(&self) -> AABB {
        self.as_ref().bbox()
    }
    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.as_ref().hit(r, t_min, t_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;
    use num_traits::Float;

    #[bench]
    fn bench_intersect_aabb_hit(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0, 0.0));
        let aabb = black_box(AABB { bounds: [point3(-1.0, -1.0, -1.0), point3(1.0, 1.0, 1.0)] });
        let t_min = black_box(0.0001);
        let t_max = black_box(f32::max_value());
        bench.iter(|| black_box(aabb.intersects(ray, t_min, t_max)));
    }

    #[bench]
    fn bench_intersect_aabb_miss(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0, 0.0));
        let aabb = black_box(AABB { bounds: [point3(-3.0, -3.0, -3.0), point3(-2.0, -2.0, -2.0)] });
        let t_min = black_box(0.0001);
        let t_max = black_box(f32::max_value());
        bench.iter(|| black_box(aabb.intersects(ray, t_min, t_max)));
    }

    #[bench]
    fn bench_intersect_aabb_hit_hit(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0, 0.0));
        let aabb_1 = black_box(AABB { bounds: [point3(-1.0, -1.0, -1.0), point3(1.0, 1.0, 1.0)] });
        let aabb_2 = black_box(aabb_1);
        let t_min = black_box(0.0001);
        let t_max = black_box(f32::max_value());
        bench.iter(|| black_box(aabb_1.intersects_2(&aabb_2, ray, t_min, t_max)));
    }

    #[bench]
    fn bench_intersect_aabb_hit_miss(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0, 0.0));
        let aabb_1 = black_box(AABB { bounds: [point3(-1.0, -1.0, -1.0), point3(1.0, 1.0, 1.0)] });
        let aabb_2 = black_box(AABB { bounds: [point3(-3.0, -3.0, -3.0), point3(-2.0, -2.0, -2.0)] });
        let t_min = black_box(0.0001);
        let t_max = black_box(f32::max_value());
        bench.iter(|| black_box(aabb_1.intersects_2(&aabb_2, ray, t_min, t_max)));
    }

    #[bench]
    fn bench_intersect_aabb_miss_miss(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0, 0.0));
        let aabb_1 = black_box(AABB { bounds: [point3(-3.0, -3.0, -3.0), point3(-2.0, -2.0, -2.0)] });
        let aabb_2 = black_box(aabb_1);
        let t_min = black_box(0.0001);
        let t_max = black_box(f32::max_value());
        bench.iter(|| black_box(aabb_1.intersects_2(&aabb_2, ray, t_min, t_max)));
    }

    #[bench]
    fn bench_merge_aabb(bench: &mut Bencher) {
        let aabb1 = black_box(AABB { bounds: [point3(-1.0, -1.0, -1.0), point3(0.0, 0.0, 0.0)] });
        let aabb2 = black_box(AABB { bounds: [point3(0.0, 0.0, 0.0), point3(1.0, 1.0, 1.0)] });
        bench.iter(|| black_box(aabb1.merge(aabb2)) );
    }
}
