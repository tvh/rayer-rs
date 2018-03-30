pub mod sphere;
pub mod triangle;
pub mod bvh;
pub mod instance;

use num_traits::Float;
use euclid::*;
use std::simd::*;

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

    const WIGGLE_FACTOR: f32 = 0.0001;

    pub fn intersects_2(&self, second: &Self, r: Ray, t0: f32, t1: f32) -> (Option<(f32, f32)>, Option<(f32, f32)>) {
        let bounds_vec_xy = f32x8::new(
            self.bounds[r.sign.x as usize].x,
            self.bounds[1-r.sign.x as usize].x,
            second.bounds[r.sign.x as usize].x,
            second.bounds[1-r.sign.x as usize].x,
            self.bounds[r.sign.y as usize].y,
            self.bounds[1-r.sign.y as usize].y,
            second.bounds[r.sign.y as usize].y,
            second.bounds[1-r.sign.y as usize].y,
        );

        let origin_x = r.origin.x;
        let origin_y = r.origin.y;
        let origin_xy = f32x8::new(
            origin_x,
            origin_x,
            origin_x,
            origin_x,
            origin_y,
            origin_y,
            origin_y,
            origin_y,
        );

        let inv_direction_x = r.inv_direction.x;
        let inv_direction_y = r.inv_direction.y;
        let inv_direction_xy = f32x8::new(
            inv_direction_x,
            inv_direction_x,
            inv_direction_x,
            inv_direction_x,
            inv_direction_y,
            inv_direction_y,
            inv_direction_y,
            inv_direction_y,
        );

        let res_vec_xy = (bounds_vec_xy - origin_xy) * inv_direction_xy;

        let mut tmin_0 = unsafe {
            let mut tmin = res_vec_xy.extract_unchecked(0);
            let tymin = res_vec_xy.extract_unchecked(4);
            if tymin>tmin {
                tmin = tymin;
            }
            tmin
        };

        let mut tmax_0 = unsafe {
            let mut tmax = res_vec_xy.extract_unchecked(1);
            let tymax = res_vec_xy.extract_unchecked(5);
            if tymax<tmax {
                tmax = tymax;
            }
            tmax
        };

        let res_0 = {
            if (tmin_0>tmax_0+AABB::WIGGLE_FACTOR) || (tmin_0 > t1) || (tmax_0 < t0) {
                None
            } else {
                let tzmin = (self.bounds[r.sign.z as usize].z - r.origin.z) * r.inv_direction.z;
                let tzmax = (self.bounds[1-r.sign.z as usize].z - r.origin.z) * r.inv_direction.z;
                if tzmin>tmin_0 {
                    tmin_0 = tzmin;
                }
                if tzmax<tmax_0 {
                    tmax_0 = tzmax;
                }
                if (tmin_0<tmax_0+AABB::WIGGLE_FACTOR) && (tmin_0 < t1) && (tmax_0 > t0) {
                    Some((tmin_0, tmax_0))
                } else {
                    None
                }
            }
        };

        let mut tmin_1 = unsafe {
            let mut tmin = res_vec_xy.extract_unchecked(2);
            let tymin = res_vec_xy.extract_unchecked(6);
            if tymin>tmin {
                tmin = tymin;
            }
            tmin
        };

        let mut tmax_1 = unsafe {
            let mut tmax = res_vec_xy.extract_unchecked(3);
            let tymax = res_vec_xy.extract_unchecked(7);
            if tymax<tmax {
                tmax = tymax;
            }
            tmax
        };

        let res_1 = {
            if (tmin_1>tmax_1+AABB::WIGGLE_FACTOR) || (tmin_1 > t1) || (tmax_1 < t0) {
                None
            } else {
                let tzmin = (second.bounds[r.sign.z as usize].z - r.origin.z) * r.inv_direction.z;
                let tzmax = (second.bounds[1-r.sign.z as usize].z - r.origin.z) * r.inv_direction.z;
                if tzmin>tmin_1 {
                    tmin_1 = tzmin;
                }
                if tzmax<tmax_1 {
                    tmax_1 = tzmax;
                }
                if (tmin_1<tmax_1+AABB::WIGGLE_FACTOR) && (tmin_1 < t1) && (tmax_1 > t0) {
                    Some((tmin_1, tmax_1))
                } else {
                    None
                }
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
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for AABB {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let l = point3(g.gen_range(-1.0, 1.0), g.gen_range(-1.0, 1.0), g.gen_range(-1.0, 1.0));
            let h = point3(l.x+g.next_f32(), l.y+g.next_f32(), l.z+g.next_f32());
            AABB { bounds: [l,h] }
        }
    }

    #[quickcheck]
    fn intersect_2_equivalence(aabb_1: AABB, aabb_2: AABB) {
        let ray = Ray::new(point3(-1.0, -1.0, -1.0), vec3(1.0, 1.0, 1.0), 500.0, 0.0);
        let t_min = 0.0001;
        let t_max = f32::max_value();
        let res_1 = aabb_1.intersects(ray, t_min, t_max);
        let res_2 = aabb_2.intersects(ray, t_min, t_max);
        assert_eq!((res_1, res_2), aabb_1.intersects_2(&aabb_2, ray, t_min, t_max));
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
