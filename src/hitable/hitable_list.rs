use ray::Ray;
use hitable::*;
use types::*;

#[allow(dead_code)]
pub struct HitableList<'a, T: 'a>(pub &'a[&'a Hitable<T>]);

impl<'a, T: CoordinateBase> Hitable<T> for HitableList<'a, T> {
    fn bbox(&self) -> AABB<T> {
        let mut bbox = AABB::<T>::empty();

        for obj in self.0.iter() {
            bbox = bbox.merge(obj.bbox());
        }
        bbox
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
