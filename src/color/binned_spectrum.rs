use color::HasReflectance;
use std::ops::*;

/// A representation over the visible spectrum using 10 bins.
/// .
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ColorSpectrum10 {
    spectrum: [f32; 10]
}

impl ColorSpectrum10 {
    pub const fn new(spectrum: [f32; 10]) -> ColorSpectrum10 {
        ColorSpectrum10{ spectrum }
    }
}

impl Add for ColorSpectrum10 {
    type Output = ColorSpectrum10;
    fn add(self, other: ColorSpectrum10) -> ColorSpectrum10 {
        let mut res = self.spectrum.clone();
        for (a, b) in other.spectrum.iter().zip(res.iter_mut()) {
            *b = *a+*b;
        }
        ColorSpectrum10 { spectrum: res }
    }
}

impl AddAssign for ColorSpectrum10 {
    fn add_assign(&mut self, other: ColorSpectrum10) {
        for (a, b) in other.spectrum.iter().zip(self.spectrum.iter_mut()) {
            *b = *a+*b;
        }
    }
}

impl Mul<ColorSpectrum10> for f32 {
    type Output = ColorSpectrum10;
    fn mul(self, other: ColorSpectrum10) -> ColorSpectrum10 {
        let mut res = other.spectrum.clone();
        for x in res.iter_mut() {
            *x *= self;
        }
        ColorSpectrum10 { spectrum: res }
    }
}

impl HasReflectance for ColorSpectrum10 {
    fn reflect(self, wl: f32) -> f32 {
        let mut index = (wl as isize-380)/34;
        if index < 0 {
            index = 0;
        }
        if index >= 10 {
            index = 9;
        }
        self.spectrum[index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;

    #[bench]
    fn bench_add(bench: &mut Bencher) {
        let a = black_box(ColorSpectrum10{ spectrum: [0.2; 10] });
        let b = black_box(ColorSpectrum10{ spectrum: [0.4; 10] });
        bench.iter(|| {
            a+b
        });
    }

    #[bench]
    fn bench_match(bench: &mut Bencher) {
        let white = black_box(ColorSpectrum10{spectrum: [1.0; 10]});
        let wl = black_box(500.0);
        bench.iter(|| white.reflect(wl));
    }
}
