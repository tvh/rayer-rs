use euclid::*;
use num_traits::float::*;

pub struct Inverted;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Ray<T> {
    pub origin: Point3D<T>,
    pub direction: Vector3D<T>,
    pub wl: f32,
    pub inv_direction: TypedVector3D<T, Inverted>,
    pub sign: TypedVector3D<bool, Inverted>,
}

impl<T> Ray<T>
where
    T: Float
{
    pub fn new(origin: Point3D<T>, direction: Vector3D<T>, wl: f32) -> Ray<T> {
        let inv_direction =
            vec3(direction.x.recip(), direction.y.recip(), direction.z.recip());
        let sign =
            vec3(direction.x < T::zero(), direction.y < T::zero(), direction.z < T::zero());
        Ray{origin, direction, wl, inv_direction, sign}
    }

    pub fn point_at_parameter(self, t: T) -> Point3D<T> {
        self.origin + self.direction*t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;

    #[bench]
    fn bench_ray_new_f32(bench: &mut Bencher) {
        let origin = black_box(point3(-3.0, 0.0, 0.0));
        let direction = black_box(vec3(1.0, 0.0, 0.0));
        let wl = black_box(500.0);
        bench.iter(|| Ray::<f32>::new(origin, direction, wl) );
    }

    #[bench]
    fn bench_ray_new_f64(bench: &mut Bencher) {
        let origin = black_box(point3(-3.0, 0.0, 0.0));
        let direction = black_box(vec3(1.0, 0.0, 0.0));
        let wl = black_box(500.0);
        bench.iter(|| Ray::<f64>::new(origin, direction, wl) );
    }
}
