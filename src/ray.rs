use euclid::*;

pub struct Inverted;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3D<f32>,
    pub direction: Vector3D<f32>,
    pub wl: f32,
    pub inv_direction: TypedVector3D<f32, Inverted>,
    pub sign: TypedVector3D<bool, Inverted>,
    pub ti: f32,
}

impl Ray
{
    pub fn new(origin: Point3D<f32>, direction: Vector3D<f32>, wl: f32, ti: f32) -> Ray {
        let inv_direction =
            vec3(direction.x.recip(), direction.y.recip(), direction.z.recip());
        let sign =
            vec3(direction.x < 0.0, direction.y < 0.0, direction.z < 0.0);
        Ray{origin, direction, wl, inv_direction, sign, ti}
    }

    pub fn point_at_parameter(self, t: f32) -> Point3D<f32> {
        self.origin + self.direction*t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;

    #[bench]
    fn bench_ray_new(bench: &mut Bencher) {
        let origin = black_box(point3(-3.0, 0.0, 0.0));
        let direction = black_box(vec3(1.0, 0.0, 0.0));
        let wl = black_box(500.0);
        bench.iter(|| black_box(Ray::new(origin, direction, wl, 0.0)) );
    }
}
