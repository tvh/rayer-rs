use euclid::*;
use std::sync::Arc;

use hitable::*;
use texture::Texture;

pub struct Triangle {
    vert: (Point3D<f32>, Point3D<f32>, Point3D<f32>),
    normal: (Vector3D<f32>, Vector3D<f32>, Vector3D<f32>),
    uv: (Vector2D<f32>, Vector2D<f32>, Vector2D<f32>),
    texture: Arc<Texture>,
}

impl Triangle {
    pub fn new(
        vert: (Point3D<f32>, Point3D<f32>, Point3D<f32>),
        normal: (Vector3D<f32>, Vector3D<f32>, Vector3D<f32>),
        uv: (Vector2D<f32>, Vector2D<f32>, Vector2D<f32>),
        texture: Arc<Texture>,
    ) -> Triangle {
        Triangle {
            vert,
            normal,
            uv,
            texture,
        }
    }
}

impl Hitable for Triangle {
    fn bbox(&self) -> AABB {
        let mut low = self.vert.0;
        let mut high = self.vert.0;
        for obj in [self.vert.1, self.vert.2].iter() {
            low = point3(
                f32::min(low.x, obj.x),
                f32::min(low.y, obj.y),
                f32::min(low.z, obj.z),
            );
            high = point3(
                f32::max(high.x, obj.x),
                f32::max(high.y, obj.y),
                f32::max(high.z, obj.z),
            );
        }
        AABB { bounds: [low, high] }
    }
    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // find vectors for two edges sharing vert0
        let edge1 = self.vert.1 - self.vert.0;
        let edge2 = self.vert.2 - self.vert.0;
        // begin calculating determinant also used to calculate U parameter
        let pvec = r.direction.cross(edge2);
        // if determinant is near zero ray lies in plane of triangle
        let det = edge1.dot(pvec);
        if !det.is_normal() {
            return None;
        }
        let inv_det = det.recip();
        // calculate distance from vert0 to ray origin
        let tvec = r.origin - self.vert.0;
        // calculate U parameter and test bounds
        let u = tvec.dot(pvec) * inv_det;
        if u<0.0 || u>1.0 {
            return None;
        }
        // prepare to test V parameter
        let qvec = tvec.cross(edge1);
        // calculate V parameter and test bounds
        let v = r.direction.dot(qvec) * inv_det;
        if v<0.0 || v>1.0 {
            return None;
        }
        // calculate t, ray intersects triangle
        let t = edge2.dot(qvec) * inv_det;
        if t<=t_min || t>=t_max {
            return None;
        }
        let w = 1.0 - u - v;
        if w<0.0 || w>1.0 {
            return None;
        }
        let normal = (self.normal.0*v + self.normal.1*u + self.normal.2*w).normalize();
        let p = r.point_at_parameter(t);
        let uv = self.uv.0*v + self.uv.1*u + self.uv.2*w;
        Some(HitRecord{p, t, normal, texture: self.texture.as_ref(), uv})
    }
}
