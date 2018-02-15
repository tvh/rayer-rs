use std::fmt::Debug;
use euclid::Vector3D;
use std::sync::Arc;

use color::HasReflectance;
use ray::Ray;
use hitable::*;
use types::*;
use random::*;
use texture::Texture;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ScatterResult<T> {
    pub emittance: f32,
    pub reflection: Option<(f32, Ray<T>)>,
}

pub trait Material<T: CoordinateBase>: Debug + Send + Sync {
    fn scatter(&self, r_in: Ray<T>, hit_record: HitRecord<T>) -> ScatterResult<T>;
}

impl<'a, 'b, T: CoordinateBase> PartialEq<Material<T>+'b> for Material<T>+'a {
    fn eq(&self, other: &(Material<T>+'b)) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(Debug, Clone)]
pub struct Lambertian<T: CoordinateBase> {
    albedo: Arc<Texture<T>>
}

impl<T: CoordinateBase> Lambertian<T> {
    pub fn new(albedo: &Arc<Texture<T>>) -> Self {
        Lambertian { albedo: albedo.clone() }
    }
}

impl<T: CoordinateBase> Material<T> for Lambertian<T> {
    fn scatter(&self, r_in: Ray<T>, rec: HitRecord<T>) -> ScatterResult<T> {
        let mut direction;
        while {
            direction = rec.normal + rand_in_unit_sphere();
            direction.square_length() < T::epsilon()*T::epsilon()
        } {}
        let ray = Ray::new(rec.p, direction, r_in.wl);
        let attenuation = self.albedo.value(rec.uv, r_in.wl);
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
pub struct Dielectric {
    b1: f32,
    b2: f32,
    b3: f32,
    c1: f32,
    c2: f32,
    c3: f32,
}

impl Dielectric {
    #[allow(dead_code)]
    pub const BAF10: Dielectric =
        Dielectric{
            b1: 1.5851495,
            b2: 0.143559385,
            b3: 1.08521269,
            c1: 0.00926681282*1e6,
            c2: 0.0424489805*1e6,
            c3: 105.613573*1e6,
        };

    #[allow(dead_code)]
    pub const SF11: Dielectric =
        Dielectric{
            b1: 1.73759695,
            b2: 0.313747346,
            b3: 1.89878101,
            c1: 0.013188707*1e6,
            c2: 0.0623068142*1e6,
            c3: 155.23629*1e6,
        };

    #[allow(dead_code)]
    pub const SF66: Dielectric =
        Dielectric{
            b1: 2.0245976,
            b2: 0.470187196,
            b3: 2.59970433,
            c1: 0.0147053225*1e6,
            c2: 0.0692998276*1e6,
            c3: 161.817601*1e6,
        };
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

impl<T: CoordinateBase> Material<T> for Dielectric {
    fn scatter(&self, r_in: Ray<T>, rec: HitRecord<T>) -> ScatterResult<T> {
        let wl_2 = r_in.wl*r_in.wl;
        let ref_idx_squared =
            1.0 +
            self.b1*wl_2/(wl_2-self.c1) +
            self.b2*wl_2/(wl_2-self.c2) +
            self.b3*wl_2/(wl_2-self.c3);
        let ref_idx: T = From::from(ref_idx_squared.sqrt());
        let (outward_normal, ni_over_nt, cosine) =
            if r_in.direction.dot(rec.normal) > T::zero() {
                (-rec.normal,
                 ref_idx,
                 ref_idx * r_in.direction.dot(rec.normal) / r_in.direction.length()
                )
            } else {
                (rec.normal,
                 ref_idx.recip(),
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
                if rand::<T>() < schlick(cosine, ref_idx) {
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
