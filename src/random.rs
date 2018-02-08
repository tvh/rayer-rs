use std::cell::RefCell;
use rand::{Rand, Rng, XorShiftRng, weak_rng};
use rand::distributions::range::SampleRange;
use euclid::*;
use num_traits::Float;

pub fn rand_in_unit_sphere<T>() -> Vector3D<T>
where T: Float + SampleRange
{
    let mut rng = thread_rng();
    let mut p: Vector3D<T>;
    let mut gen_component = || rng.gen_range(-T::one(), T::one());
    while {
        p = Vector3D::new(gen_component(), gen_component(), gen_component());
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
    rng: RefCell<XorShiftRng>,
}

thread_local!(
    static THREAD_RNG_KEY: RefCell<XorShiftRng> = {
        RefCell::new(weak_rng())
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
    use euclid::Vector3D;

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
    fn bench_rand_in_unit_sphere(bench: &mut Bencher) {
        bench.iter(|| super::rand_in_unit_sphere() as Vector3D<f32>);
    }
}
