use hitable::*;
use std::sync::Arc;
use decorum::Ordered;

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

impl<T: CoordinateBase> BVH<T> {
    pub fn initialize(items: &[Arc<Hitable<T>>]) -> BVH<T> {
        #[derive(Clone, Copy)]
        enum Axis {
            X, Y, Z
        }
        fn go<T: CoordinateBase>(items: &mut [Arc<Hitable<T>>], direction: Axis) -> BVH<T> {
            match items.len() {
                0 => { return BVH::Empty },
                1 => {
                    let item = items[0].clone();
                    let bbox = item.bbox();
                    return BVH::Tip {
                        hitable: item,
                        bbox: bbox
                    }
                },
                _ => {}
            }
            match direction {
                Axis::X => items.sort_unstable_by_key(| p | Ordered::from_inner(p.centroid().x)),
                Axis::Y => items.sort_unstable_by_key(| p | Ordered::from_inner(p.centroid().y)),
                Axis::Z => items.sort_unstable_by_key(| p | Ordered::from_inner(p.centroid().z))
            };
            let split_location = items.len()/2;
            let (mut left_items, mut right_items) = items.split_at_mut(split_location);
            let direction = match direction {
                Axis::X => Axis::Y,
                Axis::Y => Axis::Z,
                Axis::Z => Axis::X
            };
            let left = go(&mut left_items, direction);
            let right = go(&mut right_items, direction);
            let bbox = left.bbox().merge(right.bbox());
            BVH::Bin{ left: Arc::new(left), right: Arc::new(right), bbox }
        }
        let mut items: Vec<_> = items.iter().map(|x|x.clone()).collect();
        go(items.as_mut_slice(), Axis::X)
    }
}

impl<T: CoordinateBase> Hitable<T> for BVH<T> {
    fn bbox(&self) -> BoundingBox<T> {
        match self {
            &BVH::Empty => BoundingBox::<T>::empty(),
            &BVH::Bin { bbox, .. } => bbox,
            &BVH::Tip { bbox, .. } => bbox
        }
    }

    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>> {
        match self {
            &BVH::Empty => None,
            &BVH::Tip { ref hitable, bbox } => {
                if bbox.intersects(r, t_min, t_max) {
                    hitable.hit(r, t_min, t_max)
                } else {
                    None
                }
            },
            &BVH::Bin { ref left, ref right, bbox } => {
                if !bbox.intersects(r, t_min, t_max) {
                    return None;
                }
                let mut closest_match = None;
                let mut closest_so_far = t_max;

                match left.hit(r, t_min, closest_so_far) {
                    None => (),
                    Some(hit) => {
                        closest_match = Some(hit);
                        closest_so_far = hit.t;
                    }
                }
                match right.hit(r, t_min, closest_so_far) {
                    None => (),
                    Some(hit) => {
                        closest_match = Some(hit);
                    }
                }
                closest_match
            }
        }
    }
}
