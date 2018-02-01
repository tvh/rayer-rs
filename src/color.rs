use std::ops::*;
use std::iter::*;

/// A representation over the visible spectrum.
/// This has a resolution of 10nm, with index 0 representing
/// 390nm and 31 representing 700nm.
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ColorSpectrum {
    spectrum: [f32; 32]
}

impl Add for ColorSpectrum {
    type Output = ColorSpectrum;
    fn add(self, other: ColorSpectrum) -> ColorSpectrum {
        let mut res = self.spectrum.clone();
        for (a, b) in other.spectrum.iter().zip(res.iter_mut()) {
            *b = *a+*b;
        }
        ColorSpectrum { spectrum: res }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;

    #[bench]
    fn bench_add(bench: &mut Bencher) {
        let a = black_box(ColorSpectrum{ spectrum: [0.2; 32] });
        let b = black_box(ColorSpectrum{ spectrum: [0.4; 32] });
        bench.iter(|| {
            a+b
        });
    }
}
