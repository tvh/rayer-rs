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
    fn scatter(&self, r_in: Ray, _hit_record: HitRecord) -> ScatterResult {
        let emittance = self.light.reflect(r_in.wl);
        ScatterResult {
            emittance,
            reflection: None,
        }
    }
}

