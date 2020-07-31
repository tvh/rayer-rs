use std::cell::RefCell;
use std::rc::Rc;
use rand::{RngCore, Rng, SeedableRng};
use rand_xoshiro::Xoshiro256Plus;
use rand::distributions::{Distribution, Standard};
use rand::distributions::uniform::{SampleUniform};
use euclid::*;
use num_traits::Float;

pub fn rand_in_unit_sphere<T>() -> Vector3D<T>
where T: Float, Standard: Distribution<T>
{
    let mut rng = thread_rng();
    let mut p: Vector3D<T>;
    let mut gen_component = || rng.gen().mul_add(T::one()+T::one(), -T::one());
    while {
        p = vec3(gen_component(), gen_component(), gen_component());
        p.square_length() >= T::one()
    } {}
    p
}

pub fn rand_in_unit_disk<T>() -> Vector2D<T>
where T: Float, Standard: Distribution<T>
{
    let mut rng = thread_rng();
    let mut p: Vector2D<T>;
    let mut gen_component = || rng.gen().mul_add(T::one()+T::one(), -T::one());
    while {
        p = vec2(gen_component(), gen_component());
        p.square_length() >= T::one()
    } {}
    p
}

#[inline]
pub fn next_f32() -> f32 {
    rand()
}

#[inline]
pub fn rand<T>() -> T
  where Standard: Distribution<T> {
    thread_rng().gen()
}

#[inline]
/// Generate a number in a given range.
///
/// ```
/// # extern crate rayer;
/// # use rayer::random::gen_range;
/// assert_ne!(gen_range(0.0, 1.0), gen_range(0.0, 1.0));
/// assert!(gen_range(0, 5)>=0);
/// assert!(gen_range(0, 5)<5);
/// ```
pub fn gen_range<T: PartialOrd + SampleUniform>(low: T, high: T) -> T {
    thread_rng().gen_range(low, high)
}

#[derive(Clone, Debug)]
pub struct XorShiftThreadRng {
    rng: Rc<RefCell<Xoshiro256Plus>>,
}

thread_local!(
    static THREAD_RNG_KEY: Rc<RefCell<Xoshiro256Plus>> = {
        Rc::new(RefCell::new(Xoshiro256Plus::from_entropy()))
    }
);

fn thread_rng() -> XorShiftThreadRng {
    XorShiftThreadRng { rng: THREAD_RNG_KEY.with(|t| t.clone()) }
}

impl RngCore for XorShiftThreadRng {
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

    #[inline]
    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> Result<(), rand::Error> {
        self.rng.borrow_mut().try_fill_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use test::*;
    use rand;
    use rand_xorshift;
    use rand::{Rng, RngCore, SeedableRng};
    use euclid::*;

    #[bench]
    fn bench_thread_rng(bench: &mut Bencher) {
        let mut rng = rand::thread_rng();
        bench.iter(|| black_box(rng.next_u32()));
    }

    #[bench]
    fn bench_xorshift_rng(bench: &mut Bencher) {
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(2134);
        bench.iter(|| black_box(rng.next_u32()));
    }

    #[bench]
    fn bench_xorshift_thread_rng(bench: &mut Bencher) {
        let mut rng = super::thread_rng();
        bench.iter(|| black_box(rng.next_u32()));
    }

    #[bench]
    fn bench_gen_range(bench: &mut Bencher) {
        let mut rng = super::thread_rng();
        let low = black_box(-1.0);
        let high = black_box(1.0);
        bench.iter(|| black_box(rng.gen_range(low, high)));
    }

    #[bench]
    fn bench_rand_in_unit_sphere_f32(bench: &mut Bencher) {
        bench.iter(|| black_box(super::rand_in_unit_sphere() as Vector3D<f32>));
    }

    #[bench]
    fn bench_rand_in_unit_sphere_f64(bench: &mut Bencher) {
        bench.iter(|| black_box(super::rand_in_unit_sphere() as Vector3D<f64>));
    }

    #[bench]
    fn bench_rand_in_unit_disk(bench: &mut Bencher) {
        bench.iter(|| black_box(super::rand_in_unit_disk() as Vector2D<f32>));
    }
}
