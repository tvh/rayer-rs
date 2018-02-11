use hitable::*;
use std::sync::Arc;

pub enum BVH<T: CoordinateBase> {
    Empty,
    Bin {
        left: Arc<BVH<T>>,
        right: Arc<BVH<T>>,
        bbox: BoundingBox<T>,
    },
    Tip {
        hitable: Arc<Hitable<T>>,
        bbox: BoundingBox<T>,
    }
}

impl<T: CoordinateBase> Hitable<T> for BVH<T> {
    fn bbox(&self) -> BoundingBox<T> {
        match self {
            &BVH::Empty => {
                BoundingBox {
                    low: point3(T::max_value(), T::max_value(), T::max_value()),
                    high: point3(T::min_value(), T::min_value(), T::min_value()),
                }
            }
            &BVH::Bin { bbox, .. } => bbox,
            &BVH::Tip { bbox, .. } => bbox
        }
    }

    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>> {
        match self {
            &BVH::Empty => None,
            &BVH::Tip { ref hitable, .. } => {
                hitable.hit(r, t_min, t_max)
            },
            &BVH::Bin { ref left, ref right, .. } => {
                let mut closest_match = None;
                let mut closest_so_far = t_max;

                let left_bbox = left.bbox();
                if left_bbox.intersects(r, t_min, closest_so_far) {
                    match left.hit(r, t_min, closest_so_far) {
                        None => (),
                        Some(hit) => {
                            closest_match = Some(hit);
                            closest_so_far = hit.t;
                        }
                    }
                }
                let right_bbox = left.bbox();
                if right_bbox.intersects(r, t_min, closest_so_far) {
                    match right.hit(r, t_min, closest_so_far) {
                        None => (),
                        Some(hit) => {
                            closest_match = Some(hit);
                            closest_so_far = hit.t;
                        }
                    }
                }
                closest_match
            }
        }
    }
}
