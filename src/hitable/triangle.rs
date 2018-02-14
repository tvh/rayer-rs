use euclid::*;
use std::sync::Arc;
use std::borrow::Borrow;

use hitable::*;

pub struct Triangle<T: CoordinateBase> {
    vert0: Point3D<T>,
    vert1: Point3D<T>,
    vert2: Point3D<T>,
    normal0: Vector3D<T>,
    normal1: Vector3D<T>,
    normal2: Vector3D<T>,
    uv0: Vector2D<T>,
    uv1: Vector2D<T>,
    uv2: Vector2D<T>,
    material: Arc<Material<T>>,
}

impl<T: CoordinateBase> Hitable<T> for Triangle<T> {
    fn bbox(&self) -> AABB<T> {
        let mut low = self.vert0;
        let mut high = self.vert0;
        for obj in [self.vert1, self.vert2].iter() {
            low = point3(
                T::min(low.x, obj.x),
                T::min(low.y, obj.y),
                T::min(low.z, obj.z),
            );
            high = point3(
                T::max(high.x, obj.x),
                T::max(high.y, obj.y),
                T::max(high.z, obj.z),
            );
        }
        AABB { low, high }
    }
    fn hit(&self, r: Ray<T>, t_min: T, t_max: T) -> Option<HitRecord<T>> {
        // find vectors for two edges sharing vert0
        let edge1 = self.vert1 - self.vert0;
        let edge2 = self.vert2 - self.vert0;
        // begin calculating determinant also used to calculate U parameter
        let pvec = r.direction.cross(edge2);
        // if determinant is near zero ray lies in plane of triangle
        let det = edge1.dot(pvec);
        if !det.is_normal() {
            return None;
        }
        let inv_det = det.recip();
        // calculate distance from vert0 to ray origin
        let tvec = r.origin - self.vert0;
        // calculate U parameter and test bounds
        let u = tvec.dot(pvec) * inv_det;
        if u<T::zero() || u>T::one() {
            return None;
        }
        // prepare to test V parameter
        let qvec = tvec.cross(edge1);
        // calculate V parameter and test bounds
        let v = r.direction.dot(qvec) * inv_det;
        if v<T::zero() || v>T::one() {
            return None;
        }
        // calculate t, ray intersects triangle
        let t = edge2.dot(qvec) * inv_det;
        if t<=t_min || t>=t_max {
            return None;
        }
        let w = T::one() - u - v;
        if w<T::zero() || w>T::one() {
            return None;
        }
        let normal = (self.normal0*u + self.normal1*v + self.normal2*w).normalize();
        let p = r.point_at_parameter(t);
        let uv = self.uv0*u + self.uv1*v + self.uv2*w;
        Some(HitRecord{p, t, normal, material: self.material.borrow(), uv})
    }
}
