pub mod sphere;
pub mod hitable_list;

use euclid::*;

use types::*;
use ray::Ray;
use material::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct HitRecord<'a, T: 'a + CoordinateBase> {
    pub t: T,
    pub p: Point3D<T>,
    pub normal: Vector3D<T>,
    pub material: &'a Material<T>,
}

pub trait Hitable<T: CoordinateBase> {
    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>>;
}
