use std::fmt::Debug;

use color::HasReflectance;
use ray::Ray;
use hitable::*;
use types::*;
use random::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ScatterResult<T> {
    pub emittance: f32,
    pub reflection: Option<(f32, Ray<T>)>,
}

pub trait Material<T: CoordinateBase>: Debug {
    fn scatter(&self, r_in: Ray<T>, hit_record: HitRecord<T>) -> ScatterResult<T>;
}

impl<'a, 'b, T: CoordinateBase> PartialEq<Material<T>+'b> for Material<T>+'a {
    fn eq(&self, other: &(Material<T>+'b)) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Lambertian<R: HasReflectance> {
    albedo: R
}

impl<R: HasReflectance> Lambertian<R> {
    pub fn new(albedo: R) -> Self {
        Lambertian { albedo }
    }
}

impl<T: CoordinateBase, R: HasReflectance> Material<T> for Lambertian<R> {
    fn scatter(&self, r_in: Ray<T>, hit_record: HitRecord<T>) -> ScatterResult<T> {
        let direction = hit_record.normal + rand_in_unit_sphere();
        let ray = Ray::new(hit_record.p, direction, r_in.wl);
        let attenuation = self.albedo.reflect(r_in.wl);
        ScatterResult{ emittance: 0.0, reflection: Some((attenuation, ray))}
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Metal<R: HasReflectance> {
    albedo: R
}

impl<R: HasReflectance> Metal<R> {
    pub fn new(albedo: R) -> Self {
        Metal { albedo }
    }
}

impl<T: CoordinateBase, R: HasReflectance> Material<T> for Metal<R> {
    fn scatter(&self, r_in: Ray<T>, hit_record: HitRecord<T>) -> ScatterResult<T> {
        let reflected = r_in.direction - hit_record.normal*r_in.direction.dot(hit_record.normal)*T::from_f32(2.0);
        let ray = Ray::new(hit_record.p, reflected, r_in.wl);
        let attenuation = self.albedo.reflect(r_in.wl);
        ScatterResult{ emittance: 0.0, reflection: Some((attenuation, ray))}
    }
}
