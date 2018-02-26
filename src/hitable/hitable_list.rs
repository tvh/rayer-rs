use ray::Ray;
use hitable::*;

#[allow(dead_code)]
pub struct HitableList<'a>(pub &'a[&'a Hitable]);

impl<'a> Hitable for HitableList<'a> {
    fn bbox(&self) -> AABB {
        let mut bbox = AABB::empty();

        for obj in self.0.iter() {
            bbox = bbox.merge(obj.bbox());
        }
        bbox
    }
    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
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
