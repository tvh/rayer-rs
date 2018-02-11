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
    pub normal: Vector3D<T>,
    pub material: &'a Material<T>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct BoundingBox<T: CoordinateBase> {
    pub low: Point3D<T>,
    pub high: Point3D<T>,
}

impl<T: CoordinateBase> BoundingBox<T> {
    pub fn intersects(&self, r: Ray<T>, t_min: T, t_max: T) -> bool {
        let t_x0 = self.low.x - r.origin.x / r.direction.x;
        let t_x1 = self.high.x - r.origin.x / r.direction.x;
        let t_min = T::max(t_min, T::min(t_x0, t_x1));
        let t_max = T::min(t_max, T::max(t_x0, t_x1));
        let t_y0 = self.low.y - r.origin.y / r.direction.y;
        let t_y1 = self.high.y - r.origin.y / r.direction.y;
        let t_min = T::max(t_min, T::min(t_y0, t_y1));
        let t_max = T::min(t_max, T::max(t_y0, t_y1));
        let t_z0 = self.low.z - r.origin.z / r.direction.z;
        let t_z1 = self.high.z - r.origin.z / r.direction.z;
        let t_min = T::max(t_min, T::min(t_z0, t_z1));
        let t_max = T::min(t_max, T::max(t_z0, t_z1));
        t_max >= t_min
    }

    pub fn empty() -> BoundingBox<T> {
        BoundingBox {
            low: point3(T::max_value(), T::max_value(), T::max_value()),
            high: point3(T::min_value(), T::min_value(), T::min_value()),
        }
    }

    pub fn merge(self, other: BoundingBox<T>) -> BoundingBox<T> {
        let low = point3(
            T::min(self.low.x, other.low.x),
            T::min(self.low.y, other.low.y),
            T::min(self.low.z, other.low.z),
        );
        let high = point3(
            T::max(self.high.x, other.high.x),
            T::max(self.high.y, other.high.y),
            T::max(self.high.z, other.high.z),
        );
        BoundingBox { low, high }
    }
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
