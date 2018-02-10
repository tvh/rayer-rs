use std::fmt::Debug;
use euclid::Vector3D;

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
pub struct Metal<T: CoordinateBase, R: HasReflectance> {
    albedo: R,
    fuzz: T,
}

impl<R: HasReflectance, T: CoordinateBase> Metal<T, R> {
    pub fn new(albedo: R, fuzz: T) -> Self {
        let fuzz = if fuzz<T::zero() {
            T::zero()
        } else if fuzz>T::one() {
            T::one()
        } else {
            fuzz
        };
        Metal { albedo, fuzz }
    }
}

impl<T: CoordinateBase, R: HasReflectance> Material<T> for Metal<T, R> {
    fn scatter(&self, r_in: Ray<T>, hit_record: HitRecord<T>) -> ScatterResult<T> {
        let reflected = reflect(r_in.direction, hit_record.normal);
        let scattered =  reflected + rand_in_unit_sphere()*self.fuzz;
        let ray = Ray::new(hit_record.p, scattered, r_in.wl);
        let attenuation = self.albedo.reflect(r_in.wl);
        ScatterResult{ emittance: 0.0, reflection: Some((attenuation, ray))}
    }
}

fn reflect<T: CoordinateBase>(v: Vector3D<T>, n: Vector3D<T>) -> Vector3D<T> {
        v - n*v.dot(n)*From::from(2.0)
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Dielectric<T: CoordinateBase> {
    ref_idx: T,
}

impl<T: CoordinateBase> Dielectric<T> {
    pub fn new(ref_idx: T) -> Self {
        Dielectric { ref_idx }
    }
}

fn refract<T: CoordinateBase>(v: Vector3D<T>, n: Vector3D<T>, ni_over_nt: T) -> Option<Vector3D<T>> {
    let uv = v.normalize();
    let dt = uv.dot(n);
    let discriminant = T::one() - ni_over_nt*ni_over_nt*(T::one()-dt*dt);
    if discriminant > T::zero() {
        let refracted = (uv - n*dt)*ni_over_nt - n*T::sqrt(discriminant);
        Some(refracted)
    } else {
        None
    }
}

fn schlick<T: CoordinateBase>(cosine: T, ref_idx: T) -> T {
    let r0 = (T::one()-ref_idx) / (T::one()+ref_idx);
    let r0 = r0*r0;
    r0 + (T::one() - r0)*T::powf(T::one()-cosine, From::from(5.0))
}

impl<T: CoordinateBase> Material<T> for Dielectric<T> {
    fn scatter(&self, r_in: Ray<T>, rec: HitRecord<T>) -> ScatterResult<T> {
        let (outward_normal, ni_over_nt, cosine) =
            if r_in.direction.dot(rec.normal) > T::zero() {
                (-rec.normal,
                 self.ref_idx,
                 self.ref_idx * r_in.direction.dot(rec.normal) / r_in.direction.length()
                )
            } else {
                (rec.normal,
                 self.ref_idx.recip(),
                 -r_in.direction.dot(rec.normal) / r_in.direction.length()
                )
            };
        let refracted = refract(r_in.direction, outward_normal, ni_over_nt);
        let scattered = match refracted {
            None => {
                let reflected = reflect(r_in.direction, rec.normal);
                Ray::new(rec.p, reflected, r_in.wl)
            },
            Some(refracted) => {
                if rand::<T>() < schlick(cosine, self.ref_idx) {
                    let reflected = reflect(r_in.direction, rec.normal);
                    Ray::new(rec.p, reflected, r_in.wl)
                } else {
                    Ray::new(rec.p, refracted, r_in.wl)
                }
            }
        };
        ScatterResult{ emittance: 0.0, reflection: Some((1.0, scattered)) }

    }
}
