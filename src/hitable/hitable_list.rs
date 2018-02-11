use ray::Ray;
use hitable::*;
use types::*;

pub struct HitableList<'a, T: 'a>(pub &'a[&'a Hitable<T>]);

impl<'a, T: CoordinateBase> Hitable<T> for HitableList<'a, T> {
    fn bbox(&self) -> BoundingBox<T> {
        let mut low = point3(T::max_value(), T::max_value(), T::max_value());
        let mut high = point3(T::min_value(), T::min_value(), T::min_value());

        for obj in self.0.iter() {
            let obj_bbox = obj.bbox();
            low = point3(
                T::min(low.x, obj_bbox.low.x),
                T::min(low.y, obj_bbox.low.y),
                T::min(low.z, obj_bbox.low.z),
            );
            high = point3(
                T::max(high.x, obj_bbox.high.x),
                T::max(high.y, obj_bbox.high.y),
                T::max(high.z, obj_bbox.high.z),
            );
        }
        BoundingBox { low, high }
    }
    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>> {
        let mut closest_match = None;
        let mut closest_so_far = t_max;
        for obj in self.0.iter() {
            match obj.hit(r, t_min, closest_so_far) {
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
