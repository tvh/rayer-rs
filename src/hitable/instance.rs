use euclid::*;
use hitable::*;
use ray::*;
use num_traits::FloatConst;

/// Translate a given object
///
/// ```
/// # extern crate rayer;
/// # extern crate palette;
/// # extern crate euclid;
/// # use euclid::*;
/// # use palette::*;
/// # use std::sync::Arc;
/// # use rayer::texture::*;
/// # use rayer::material::*;
/// # use rayer::hitable::*;
/// # use rayer::hitable::instance::Translate;
/// # use rayer::hitable::triangle::axis_aligned_cuboid;
/// #
/// # let offset = vec3(1.0, 2.0, 3.0);
/// # let texture: Arc<Texture> = Arc::new(Lambertian::new(Rgb::with_wp(0.5, 0.5, 0.5)));
/// # let object = axis_aligned_cuboid(point3(-10.0, -100.0, -1000.0), point3(10.0, 100.0, 1000.0), texture);
/// let translated_object = Translate { offset, object: object.clone() };
/// assert_eq!(translated_object.bbox().bounds[0], object.bbox().bounds[0]+offset);
/// assert_eq!(translated_object.bbox().bounds[1], object.bbox().bounds[1]+offset);
/// ```
#[derive(Debug, Clone)]
pub struct Translate<H: Hitable> {
    pub object: H,
    pub offset: Vector3D<f32>,
}

pub fn translate<H: Hitable>(
    object: H,
    offset: Vector3D<f32>,
) -> impl Hitable {
    Translate {
        offset,
        object
    }
}

impl<H: Hitable> Hitable for Translate<H> {
    fn bbox(&self) -> AABB {
        match self.object.bbox() {
            AABB { bounds: [l,h] } => AABB { bounds: [l+self.offset, h+self.offset] }
        }
    }

    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let r = Ray {
            origin: r.origin-self.offset,
            ..r
        };
        let res = self.object.hit(r, t_min, t_max);
        match res {
            None => None,
            Some(rec) => {
                Some(HitRecord{
                    p: rec.p+self.offset,
                    ..rec
                })
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RotateY<H: Hitable> {
    sin_theta: f32,
    cos_theta: f32,
    object: H,
    bbox: AABB,
}

impl<H: Hitable> Hitable for RotateY<H> {
    fn bbox(&self) -> AABB {
        self.bbox
    }
    fn hit(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut origin = r.origin;
        origin.x = self.cos_theta*r.origin.x - self.sin_theta*r.origin.z;
        origin.z = self.sin_theta*r.origin.x + self.cos_theta*r.origin.z;
        let mut direction = r.direction;
        direction.x = self.cos_theta*r.direction.x - self.sin_theta*r.direction.z;
        direction.z = self.sin_theta*r.direction.x + self.cos_theta*r.direction.z;
        let rotated_r =
            Ray::new(
                origin,
                direction,
                r.wl,
                r.ti
            );
        match self.object.hit(rotated_r, t_min, t_max) {
            None => None,
            Some(rec) => {
                let mut p = rec.p;
                p.x = self.cos_theta*rec.p.x + self.sin_theta*rec.p.z;
                p.z = -self.sin_theta*rec.p.x + self.cos_theta*rec.p.z;
                let mut normal = rec.normal;
                normal.x = self.cos_theta*rec.normal.x + self.sin_theta*rec.normal.z;
                normal.z = -self.sin_theta*rec.normal.x + self.cos_theta*rec.normal.z;
                Some(HitRecord{
                    p,
                    normal,
                    ..rec
                })
            }
        }
    }
}

pub fn rotate_y<H: Hitable>(object: H, angle: f32) -> impl Hitable {
    let theta = (f32::PI()/180.0) * angle;
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    let object_bbox = object.bbox();
    let mut bbox = AABB::empty();
    if object_bbox.is_empty() {
        return RotateY { cos_theta, sin_theta, object, bbox }
    }
    for i in 0..2 {
        let x = object_bbox.bounds[i].x;
        for j in 0..2 {
            let y = object_bbox.bounds[j].y;
            for k in 0..2 {
                let z = object_bbox.bounds[k].z;
                let newx = cos_theta*x + sin_theta*z;
                let newz = -sin_theta*x + cos_theta*z;
                let p = point3(newx, y, newz);
                bbox = bbox.merge(AABB { bounds: [p,p] })
            }
        }
    }
    RotateY { cos_theta, sin_theta, object, bbox }
}
