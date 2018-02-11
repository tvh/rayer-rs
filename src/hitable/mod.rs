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
pub enum BoundingBox<T: CoordinateBase> {
    Empty,
    NonEmpty{
        low: Point3D<T>,
        high: Point3D<T>,
    }
}

impl<T: CoordinateBase> BoundingBox<T> {
    pub fn intersects(&self, r: Ray<T>, t_min: T, t_max: T) -> bool {
        match self {
            &BoundingBox::Empty => false,
            &BoundingBox::NonEmpty { low, high } => {
                let t_x0 = (low.x - r.origin.x) / r.direction.x;
                let t_x1 = (high.x - r.origin.x) / r.direction.x;
                let t_min = T::max(t_min, T::min(t_x0, t_x1));
                let t_max = T::min(t_max, T::max(t_x0, t_x1));
                let t_y0 = (low.y - r.origin.y) / r.direction.y;
                let t_y1 = (high.y - r.origin.y) / r.direction.y;
                let t_min = T::max(t_min, T::min(t_y0, t_y1));
                let t_max = T::min(t_max, T::max(t_y0, t_y1));
                let t_z0 = (low.z - r.origin.z) / r.direction.z;
                let t_z1 = (high.z - r.origin.z) / r.direction.z;
                let t_min = T::max(t_min, T::min(t_z0, t_z1));
                let t_max = T::min(t_max, T::max(t_z0, t_z1));
                t_max >= t_min
            }
        }
    }

    pub fn empty() -> BoundingBox<T> {
        BoundingBox::Empty
    }

    pub fn merge(self, other: BoundingBox<T>) -> BoundingBox<T> {
        match (self, other) {
            (BoundingBox::Empty, _) => other,
            (_, BoundingBox::Empty) => self,
            (BoundingBox::NonEmpty { low: low_0, high: high_0 }, BoundingBox::NonEmpty { low: low_1, high: high_1 }) => {
                let low = point3(
                    T::min(low_0.x, low_1.x),
                    T::min(low_0.y, low_1.y),
                    T::min(low_0.z, low_1.z),
                );
                let high = point3(
                    T::max(high_0.x, high_1.x),
                    T::max(high_0.y, high_1.y),
                    T::max(high_0.z, high_1.z),
                );
                BoundingBox::NonEmpty { low, high }
            }
        }
    }
}

pub trait Hitable<T: CoordinateBase> {
    fn centroid(&self) -> Point3D<T> {
        let bbox = self.bbox();
        match bbox {
            BoundingBox::Empty => point3(T::max_value(), T::max_value(), T::max_value()),
            BoundingBox::NonEmpty{ low, high } => {
                point3(
                    (low.x + high.x)*From::from(0.5),
                    (low.y + high.y)*From::from(0.5),
                    (low.z + high.z)*From::from(0.5)
                )
            }
        }
    }
    fn bbox(&self) -> BoundingBox<T>;
    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>>;
}
