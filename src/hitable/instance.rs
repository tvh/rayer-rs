use euclid::*;
use hitable::*;
use ray::*;

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
    pub offset: Vector3D<f32>,
    pub object: H
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
