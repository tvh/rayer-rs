use std::fmt::Debug;
use euclid::Vector3D;

use color::HasReflectance;
use ray::Ray;
use hitable::*;
use random::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ScatterResult {
    pub emittance: f32,
    pub reflection: Option<(f32, Ray)>,
}

pub trait Material: Debug + Send + Sync {
    fn scatter(&self, r_in: Ray, hit_record: HitRecord) -> ScatterResult;
}

impl<'a, 'b> PartialEq<Material+'b> for Material+'a {
    fn eq(&self, other: &(Material+'b)) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(Debug, Clone)]
pub struct Lambertian<C: HasReflectance> {
    albedo: C
}

impl<C: HasReflectance> Lambertian<C> {
    pub fn new(albedo: C) -> Self {
        Lambertian { albedo }
    }
}

impl<C: HasReflectance> Material for Lambertian<C> {
    fn scatter(&self, r_in: Ray, rec: HitRecord) -> ScatterResult {
        let direction = rec.normal + rand_in_unit_sphere();
        let ray = Ray::new(rec.p, direction, r_in.wl);
        let attenuation = self.albedo.reflect(r_in.wl);
        ScatterResult{ emittance: 0.0, reflection: Some((attenuation, ray))}
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Metal<R: HasReflectance> {
    albedo: R,
    fuzz: f32,
}

impl<R: HasReflectance> Metal<R> {
    pub fn new(albedo: R, fuzz: f32) -> Self {
        let fuzz = if fuzz<0.0 {
            0.0
        } else if fuzz>1.0 {
            1.0
        } else {
            fuzz
        };
        Metal { albedo, fuzz }
    }
}

impl<R: HasReflectance> Material for Metal<R> {
    fn scatter(&self, r_in: Ray, hit_record: HitRecord) -> ScatterResult {
        let reflected = reflect(r_in.direction, hit_record.normal);
        let scattered =  reflected + rand_in_unit_sphere()*self.fuzz;
        let ray = Ray::new(hit_record.p, scattered, r_in.wl);
        let attenuation = self.albedo.reflect(r_in.wl);
        ScatterResult{ emittance: 0.0, reflection: Some((attenuation, ray))}
    }
}

fn reflect(v: Vector3D<f32>, n: Vector3D<f32>) -> Vector3D<f32> {
    v - n*v.dot(n)*2.0
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

fn refract(v: Vector3D<f32>, n: Vector3D<f32>, ni_over_nt: f32) -> Option<Vector3D<f32>> {
    let uv = v.normalize();
    let dt = uv.dot(n);
    let discriminant = 1.0 - ni_over_nt*ni_over_nt*(1.0-dt*dt);
    if discriminant > 0.0 {
        let refracted = (uv - n*dt)*ni_over_nt - n*f32::sqrt(discriminant);
        Some(refracted)
    } else {
        None
    }
}

fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0-ref_idx) / (1.0+ref_idx);
    let r0 = r0*r0;
    r0 + (1.0 - r0)*f32::powi(1.0-cosine, 5)
}

impl Material for Dielectric {
    fn scatter(&self, r_in: Ray, rec: HitRecord) -> ScatterResult {
        let wl_2 = r_in.wl*r_in.wl;
        let ref_idx_squared =
            1.0 +
            self.b1*wl_2/(wl_2-self.c1) +
            self.b2*wl_2/(wl_2-self.c2) +
            self.b3*wl_2/(wl_2-self.c3);
        let ref_idx = ref_idx_squared.sqrt();
        let (outward_normal, ni_over_nt, cosine) =
            if r_in.direction.dot(rec.normal) > 0.0 {
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
                if next_f32() < schlick(cosine, ref_idx) {
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
