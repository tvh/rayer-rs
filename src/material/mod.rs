use std::fmt::Debug;

use ray::Ray;
use hitable::*;
use types::*;
use random::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ScatterResult<T> {
    pub emittance: f32,
    pub reflection: Option<(f32, Ray<T>)>,
}

pub trait Material<T: CoordinateBase>: Debug {
    fn scatter(&self, r_in: Ray<T>, hit_record: HitRecord<T>) -> ScatterResult<T>;
}

impl<'a, 'b, T: CoordinateBase> PartialEq<Material<T>+'b> for Material<T>+'a {
    fn eq(&self, other: &(Material<T>+'b)) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Lambertian;

impl Default for Lambertian {
    fn default() -> Lambertian { Lambertian }
}

impl<T: CoordinateBase> Material<T> for Lambertian {
    fn scatter(&self, r_in: Ray<T>, hit_record: HitRecord<T>) -> ScatterResult<T> {
        let direction = hit_record.normal + rand_in_unit_sphere();
        let ray = Ray::new(hit_record.p, direction, r_in.wl);
        ScatterResult{ emittance: 0.0, reflection: Some((0.5, ray))}
    }
}
