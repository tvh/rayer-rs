use std::cell::RefCell;
use std::rc::Rc;
use rand::{Rand, Rng, XorShiftRng, weak_rng};
use rand::distributions::range::SampleRange;
use euclid::*;
use num_traits::Float;

pub fn rand_in_unit_sphere<T>() -> Vector3D<T>
where T: Float + Rand
{
    let mut rng = thread_rng();
    let mut p: Vector3D<T>;
    let mut gen_component = || T::rand(&mut rng).mul_add(T::one()+T::one(), -T::one());
    while {
        p = vec3(gen_component(), gen_component(), gen_component());
        p.dot(p) >= T::one()
    } {}
    p
}

pub fn rand_in_unit_disk<T>() -> Vector2D<T>
where T: Float + Rand
{
    let mut rng = thread_rng();
    let mut p: Vector2D<T>;
    let mut gen_component = || T::rand(&mut rng).mul_add(T::one()+T::one(), -T::one());
    while {
        p = vec2(gen_component(), gen_component());
        p.dot(p) >= T::one()
    } {}
    p
}

#[inline]
pub fn next_f32() -> f32 {
    rand()
}

#[inline]
pub fn rand<T: Rand>() -> T {
    T::rand(&mut thread_rng())
}

#[inline]
pub fn gen_range<T: PartialOrd + SampleRange>(low: T, high: T) -> T {
    thread_rng().gen_range(low, high)
}

#[derive(Clone, Debug)]
pub struct XorShiftThreadRng {
    rng: Rc<RefCell<XorShiftRng>>,
}

thread_local!(
    static THREAD_RNG_KEY: Rc<RefCell<XorShiftRng>> = {
        Rc::new(RefCell::new(weak_rng()))
    }
);

fn thread_rng() -> XorShiftThreadRng {
    XorShiftThreadRng { rng: THREAD_RNG_KEY.with(|t| t.clone()) }
}

impl Rng for XorShiftThreadRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.rng.borrow_mut().next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.rng.borrow_mut().next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        self.rng.borrow_mut().fill_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use test::*;
    use rand;
    use rand::Rng;
    use euclid::*;

    #[test]
    fn test_gen_range() {
        let res1 = super::gen_range(-1.0, 1.0);
        let res2 = super::gen_range(-1.0 as f32, 1.0);
        assert!(res1 != res2);
    }

    #[bench]
    fn bench_thread_rng(bench: &mut Bencher) {
        let mut rng = rand::thread_rng();
        bench.iter(|| rng.next_f32());
    }

    #[bench]
    fn bench_xorshift_rng(bench: &mut Bencher) {
        let mut rng = rand::XorShiftRng::new_unseeded();
        bench.iter(|| rng.next_f32());
    }

    #[bench]
    fn bench_xorshift_thread_rng(bench: &mut Bencher) {
        let mut rng = super::thread_rng();
        bench.iter(|| rng.next_f32());
    }

    #[bench]
    fn bench_gen_range(bench: &mut Bencher) {
        let mut rng = super::thread_rng();
        let low = black_box(-1.0);
        let high = black_box(1.0);
        bench.iter(|| rng.gen_range(low, high));
    }

    #[bench]
    fn bench_rand_in_unit_sphere_f32(bench: &mut Bencher) {
        bench.iter(|| super::rand_in_unit_sphere() as Vector3D<f32>);
    }

    #[bench]
    fn bench_rand_in_unit_sphere_f64(bench: &mut Bencher) {
        bench.iter(|| super::rand_in_unit_sphere() as Vector3D<f64>);
    }

    #[bench]
    fn bench_rand_in_unit_disk(bench: &mut Bencher) {
        bench.iter(|| super::rand_in_unit_disk() as Vector2D<f32>);
    }
}
