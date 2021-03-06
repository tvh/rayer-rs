use std::marker::PhantomData;
use std::ops::*;
use std::fmt::Debug;
use core::array::FixedSizeArray;
use std::fmt;
use palette::*;
use palette::white_point::E;

use color::cie_1931::xyz_from_wavelength;
use color::HasReflectance;

pub trait BinData: Send + Sync {
    type Spectrum: Clone + Copy + FixedSizeArray<f32> + Send + Sync;
    const WL_0: f32;
    const BIN_WIDTH: f32;
}

#[derive(Debug)]
pub struct Bin36;
impl BinData for Bin36 {
    type Spectrum = [f32; 36];
    const WL_0: f32 = 360.0;
    const BIN_WIDTH: f32 = 10.0;
}

/// The standard spectrum type used
pub type ColorSpectrum = BinnedSpectrum<Bin36>;

/// A binned representation of the visible spectrum.
/// Values outside this range are clamped to the nearest index.
pub struct BinnedSpectrum<T: BinData> {
    spectrum: T::Spectrum,
    marker: PhantomData<T>
}

impl<T: BinData> PartialEq for BinnedSpectrum<T> {
    fn eq(&self, right: &Self) -> bool {
        let mut res = true;
        for (&x,&y) in self.spectrum.as_slice().iter().zip(right.spectrum.as_slice().iter()) {
            res &= x==y;
        }
        return res;
    }
}

impl<T: BinData> Debug for BinnedSpectrum<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BinnedSpectrum")
            .field("wl_0", &T::WL_0)
            .field("bin_width", &T::BIN_WIDTH)
            .field("spectrum", &self.spectrum.as_slice())
            .finish()
    }
}

impl<T: BinData> BinnedSpectrum<T> {
    pub const fn new(spectrum: T::Spectrum) -> BinnedSpectrum<T> {
        BinnedSpectrum{ spectrum, marker: PhantomData }
    }
}

impl<T> Copy for BinnedSpectrum<T> where
    T: BinData
{}
impl<T> Clone for BinnedSpectrum<T> where
    T: BinData
{
    fn clone(&self) -> BinnedSpectrum<T> {
        *self
    }
}

impl<T: BinData> Add for BinnedSpectrum<T> {
    type Output = BinnedSpectrum<T>;
    fn add(self, other: BinnedSpectrum<T>) -> BinnedSpectrum<T> {
        let mut res = self.spectrum.clone();
        for (a, b) in other.spectrum.as_slice().iter().zip(res.as_mut_slice().iter_mut()) {
            *b = *a+*b;
        }
        BinnedSpectrum::new(res)
    }
}

impl<T: BinData> AddAssign for BinnedSpectrum<T> {
    fn add_assign(&mut self, other: BinnedSpectrum<T>) {
        for (a, b) in other.spectrum.as_slice().iter().zip(self.spectrum.as_mut_slice().iter_mut()) {
            *b = *a+*b;
        }
    }
}

impl<T: BinData> Mul<BinnedSpectrum<T>> for f32 {
    type Output = BinnedSpectrum<T>;
    fn mul(self, other: BinnedSpectrum<T>) -> BinnedSpectrum<T> {
        let mut res = other.spectrum.clone();
        for x in res.as_mut_slice().iter_mut() {
            *x *= self;
        }
        BinnedSpectrum::new(res)
    }
}

impl<T: BinData> HasReflectance for BinnedSpectrum<T> {
    fn reflect(&self, wl: f32) -> f32 {
        let mut index: isize = ((wl-T::WL_0)/T::BIN_WIDTH) as isize;
        if index < 0 {
            index = 0;
        }
        let len = self.spectrum.as_slice().len() as isize;
        if index >= len {
            index = len-1;
        }
        self.spectrum.as_slice()[index as usize]
    }

    fn reflect_xyz(&self) -> Xyz<E, f32> {
        let mut res = Xyz::with_wp(0.0, 0.0, 0.0);
        let mut wl = T::WL_0;
        for &v in self.spectrum.as_slice().iter() {
            res = res+(xyz_from_wavelength(wl)*v);
            wl += T::BIN_WIDTH;
        }
        res = res*3.0/(self.spectrum.as_slice().len() as f32);
        return res;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;

    #[derive(Debug)]
    struct Bin10;
    impl BinData for Bin10 {
        type Spectrum = [f32; 10];
        const WL_0: f32 = 380.0;
        const BIN_WIDTH: f32 = 34.0;
    }

    type ColorSpectrum10 = BinnedSpectrum<Bin10>;

    #[bench]
    fn bench_add(bench: &mut Bencher) {
        let a = black_box(ColorSpectrum10::new([0.2; 10]));
        let b = black_box(ColorSpectrum10::new([0.4; 10]));
        bench.iter(|| {
            black_box(a+b)
        });
    }

    #[bench]
    fn bench_match(bench: &mut Bencher) {
        let white = black_box(ColorSpectrum10::new([1.0; 10]));
        let wl = black_box(500.0);
        bench.iter(|| black_box(white.reflect(wl)));
    }
}
