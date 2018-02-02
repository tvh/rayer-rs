use std::ops::*;
use std::iter::*;
use palette::*;

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

/// Construct a color in XYZ from a single wavelength.
/// The algorithm is taken from
/// "Simple analytic approximations to the CIE XYZ color matching functions"
/// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.439.3537&rep=rep1&type=pdf
fn xyz_from_wavelength(wl: f32) -> Xyz<f32> {
    let tmp1_x = f32::ln((wl-570.1)/1014.0);
    let tmp2_x = f32::ln((1338.0-wl)/743.5);
    let x = 0.398*f32::exp(-1250.0*tmp1_x*tmp1_x)
        + 1.132*f32::exp(-234.0*tmp2_x*tmp2_x);

    let tmp_y = (wl-556.1)/46.14;
    let y = 1.011*f32::exp(-0.5*tmp_y*tmp_y);

    let tmp_z = f32::ln((wl-256.8)/180.4);
    let z = 2.060*f32::exp(-32.0*tmp_z*tmp_z);

    Xyz::new(x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;

    #[test]
    fn test_xyz_from_wavelength() {
        assert_eq!(xyz_from_wavelength(600.0), Xyz::new(0.0, 0.0, 0.0));
    }

    #[bench]
    fn bench_xyz_from_wavelength(bench: &mut Bencher) {
        let a = black_box(500.0);
        bench.iter(|| xyz_from_wavelength(a));
    }

    #[bench]
    fn bench_xyz_to_rgb(bench: &mut Bencher) {
        let a = black_box(xyz_from_wavelength(500.0));
        bench.iter(|| Rgb::from_xyz(a));
    }

    #[bench]
    fn bench_add(bench: &mut Bencher) {
        let a = black_box(ColorSpectrum{ spectrum: [0.2; 32] });
        let b = black_box(ColorSpectrum{ spectrum: [0.4; 32] });
        bench.iter(|| {
            a+b
        });
    }
}
