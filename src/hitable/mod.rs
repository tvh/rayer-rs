pub mod sphere;
pub mod triangle;
pub mod bvh;
pub mod instance;

use num_traits::Float;
use euclid::*;
use packed_simd::*;

use ray::*;
use texture::*;

#[derive(PartialEq, Debug)]
pub struct HitRecord<'a> {
    pub t: f32,
    pub p: Point3D<f32>,
    pub uv: Vector2D<f32>,
    pub normal: Vector3D<f32>,
    pub texture: &'a dyn Texture,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct AABB {
    pub bounds: [Point3D<f32>;2]
}

impl AABB {
    pub fn intersects(&self, r: Ray, t0: f32, t1: f32) -> Option<f32> {
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
                    Some(tmin)
                } else {
                    None
                }
            }
        }
    }

    pub fn prepare_intersect(r: Ray) -> (f32x4, f32x4, TypedVector3D<bool, Inverted>) {
        let origin_vec = f32x4::new(
            r.origin.x,
            r.origin.y,
            r.origin.z,
            r.origin.z,
        );

        let inv_direction_vec = f32x4::new(
            r.inv_direction.x,
            r.inv_direction.y,
            r.inv_direction.z,
            r.inv_direction.z,
        );

        return (origin_vec, inv_direction_vec, r.sign)
    }

    const WIGGLE_FACTOR: f32 = 0.0001;

    #[inline(always)]
    pub fn intersects_2(&self, second: &Self, sign: TypedVector3D<bool, Inverted>, origin_vec: f32x4, inv_direction_vec: f32x4, t0: f32, t1: f32) -> (Option<f32>, Option<f32>) {
        let tmin_0 = {
            let bounds_vec = f32x4::new(
                self.bounds[sign.x as usize].x,
                self.bounds[sign.y as usize].y,
                self.bounds[sign.z as usize].z,
                self.bounds[sign.z as usize].z,
            );
            let res_vec = (bounds_vec - origin_vec) * inv_direction_vec;
            res_vec.max_element()
        };

        let tmax_0 = {
            let bounds_vec = f32x4::new(
                self.bounds[1-sign.x as usize].x,
                self.bounds[1-sign.y as usize].y,
                self.bounds[1-sign.z as usize].z,
                self.bounds[1-sign.z as usize].z,
            );
            let res_vec = (bounds_vec - origin_vec) * inv_direction_vec;
            res_vec.min_element()
        };

        let tmin_1 = {
            let bounds_vec = f32x4::new(
                second.bounds[sign.x as usize].x,
                second.bounds[sign.y as usize].y,
                second.bounds[sign.z as usize].z,
                second.bounds[sign.z as usize].z,
            );
            let res_vec = (bounds_vec - origin_vec) * inv_direction_vec;
            res_vec.max_element()
        };

        let tmax_1 = {
            let bounds_vec = f32x4::new(
                second.bounds[1-sign.x as usize].x,
                second.bounds[1-sign.y as usize].y,
                second.bounds[1-sign.z as usize].z,
                second.bounds[1-sign.z as usize].z,
            );
            let res_vec = (bounds_vec - origin_vec) * inv_direction_vec;
            res_vec.min_element()
        };

        let res_0 = {
            if (tmin_0>tmax_0+AABB::WIGGLE_FACTOR) || (tmin_0 > t1) || (tmax_0 < t0) {
                None
            } else {
                Some(tmin_0)
            }
        };

        let res_1 = {
            if (tmin_1>tmax_1+AABB::WIGGLE_FACTOR) || (tmin_1 > t1) || (tmax_1 < t0) {
                None
            } else {
                Some(tmin_1)
            }
        };

        (res_0, res_1)
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

impl<T: AsRef<dyn Hitable> + Sync + Send> Hitable for T {
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
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for AABB {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let gen_range = |g: &mut G| f32::arbitrary(g)*2.0 - 1.0;
            let l = point3(gen_range(g), gen_range(g), gen_range(g));
            let h = point3(l.x+f32::arbitrary(g), l.y+f32::arbitrary(g), l.z+f32::arbitrary(g));
            AABB { bounds: [l,h] }
        }
    }

    quickcheck ! {
        fn intersect_2_equivalence(aabb_1: AABB, aabb_2: AABB) -> () {
            let ray = Ray::new(point3(-1.0, -1.0, -1.0), vec3(1.0, 1.0, 1.0), 500.0, 0.0);
            let t_min = 0.0001;
            let t_max = f32::max_value();
            let res_1 = aabb_1.intersects(ray, t_min, t_max);
            let res_2 = aabb_2.intersects(ray, t_min, t_max);
            let (origin_vec, inv_direction_vec, sign) = AABB::prepare_intersect(ray);
            assert_eq!((res_1, res_2), aabb_1.intersects_2(&aabb_2, sign, origin_vec, inv_direction_vec, t_min, t_max));
        }
    }

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
        let (origin_vec, inv_direction_vec, sign) = AABB::prepare_intersect(ray);
        bench.iter(|| black_box(aabb_1.intersects_2(&aabb_2, sign, origin_vec, inv_direction_vec, t_min, t_max)));
    }

    #[bench]
    fn bench_intersect_aabb_hit_miss(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0, 0.0));
        let aabb_1 = black_box(AABB { bounds: [point3(-1.0, -1.0, -1.0), point3(1.0, 1.0, 1.0)] });
        let aabb_2 = black_box(AABB { bounds: [point3(-3.0, -3.0, -3.0), point3(-2.0, -2.0, -2.0)] });
        let t_min = black_box(0.0001);
        let t_max = black_box(f32::max_value());
        let (origin_vec, inv_direction_vec, sign) = AABB::prepare_intersect(ray);
        bench.iter(|| black_box(aabb_1.intersects_2(&aabb_2, sign, origin_vec, inv_direction_vec, t_min, t_max)));
    }

    #[bench]
    fn bench_intersect_aabb_miss_miss(bench: &mut Bencher) {
        let ray = black_box(Ray::new(point3(-3.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), 500.0, 0.0));
        let aabb_1 = black_box(AABB { bounds: [point3(-3.0, -3.0, -3.0), point3(-2.0, -2.0, -2.0)] });
        let aabb_2 = black_box(aabb_1);
        let t_min = black_box(0.0001);
        let t_max = black_box(f32::max_value());
        let (origin_vec, inv_direction_vec, sign) = AABB::prepare_intersect(ray);
        bench.iter(|| black_box(aabb_1.intersects_2(&aabb_2, sign, origin_vec, inv_direction_vec, t_min, t_max)));
    }

    #[bench]
    fn bench_merge_aabb(bench: &mut Bencher) {
        let aabb1 = black_box(AABB { bounds: [point3(-1.0, -1.0, -1.0), point3(0.0, 0.0, 0.0)] });
        let aabb2 = black_box(AABB { bounds: [point3(0.0, 0.0, 0.0), point3(1.0, 1.0, 1.0)] });
        bench.iter(|| black_box(aabb1.merge(aabb2)) );
    }
}
