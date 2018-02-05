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
pub fn xyz_from_wavelength(wl: f32) -> Xyz<f32> {
    let tmp1_x = f32::ln((wl+570.1)/1014.0);
    let tmp2_x = f32::ln((1338.0-wl)/743.5);
    let x = 0.398*f32::exp(-1250.0*tmp1_x*tmp1_x)
        + 1.132*f32::exp(-234.0*tmp2_x*tmp2_x);

    let tmp_y = (wl-556.1)/46.14;
    let y = 1.011*f32::exp(-0.5*tmp_y*tmp_y);

    let tmp_z = f32::ln((wl-265.8)/180.4);
    let z = 2.060*f32::exp(-32.0*tmp_z*tmp_z);

    Xyz::new(x, y, z)
}

/// Just does a naive translation from the color to a matching function.
/// TODO: Use "An RGB-to-spectrum conversion for reflectances" instead
pub fn match_color<C: IntoColor<f32>>(color: C, wl: f32) -> f32 {
    let color_xyz = color.into_xyz();
    let wl_color = xyz_from_wavelength(wl);
    let wl_sum = wl_color.x + wl_color.y + wl_color.z;
    let wl_color_normalized = wl_color/wl_sum;
    let tmp = wl_color_normalized*color_xyz;
    tmp.x+tmp.y+tmp.z
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;
    use palette::*;

    #[test]
    fn test_xyz_from_valid() {
        for i in 300..800 {
            let xyz = xyz_from_wavelength(i as f32);
            let xyz_clamped = xyz.clamp();
            assert_eq!(xyz, xyz_clamped, "Invalid xyz for wl={:}", i);
        }
    }

    #[test]
    fn test_match_color_rgb() {
        let white = Rgb::new(1.0, 1.0, 1.0);
        for i in 300..800 {
            let val = match_color(white, i as f32);
            assert!((val - 1.0).abs()<0.07
                    ,"White didn't match close to 1 for {:}nm, got instead: {:}"
                    , i, val
            );
        }
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
