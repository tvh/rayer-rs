use material::*;

use color::HasReflectance;
use ray::Ray;
use hitable::*;

#[derive(Debug, Clone)]
pub struct DiffuseLight<C: HasReflectance> {
    light: C,
}

impl<C: HasReflectance> DiffuseLight<C> {
    pub fn new(light: C) -> Self {
        DiffuseLight { light }
    }
}

impl<C: HasReflectance> Material for DiffuseLight<C> {
    fn scatter(&self, r_in: Ray, _hit_record: HitRecord) -> Vec<ScatterResult> {
        let emittance =
            r_in.wl.iter().map(|&wl| (self.light.reflect(wl),0.0)).collect();
        vec![ScatterResult {
            emittance,
            reflection: None,
        }]
    }
}

