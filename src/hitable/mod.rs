pub mod sphere;
pub mod hitable_list;

use euclid::*;

use ray::Ray;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct HitRecord<T> {
    pub t: T,
    pub p: Point3D<T>,
    pub normal: Vector3D<T>
}

pub trait Hitable<T> {
    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>>;
}
