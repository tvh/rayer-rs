pub mod sphere;
pub mod hitable_list;
pub mod triangle;

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

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct BoundingBox<T: CoordinateBase> {
    pub low: Point3D<T>,
    pub high: Point3D<T>,
}

pub trait Hitable<T: CoordinateBase> {
    fn centroid(&self) -> Point3D<T> {
        let bbox = self.bbox();
        point3(
            (bbox.low.x + bbox.high.x)*From::from(0.5),
            (bbox.low.y + bbox.high.y)*From::from(0.5),
            (bbox.low.z + bbox.high.z)*From::from(0.5)
        )
    }
    fn bbox(&self) -> BoundingBox<T>;
    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>>;
}
