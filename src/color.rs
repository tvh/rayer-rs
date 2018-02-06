use std::ops::*;
use std::iter::*;
use palette::*;

pub trait HasReflectance {
    fn reflect(self, wl: f32) -> f32;
}

/// A representation over the visible spectrum using 10 bins.
/// .
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ColorSpectrum10 {
    spectrum: [f32; 10]
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

mod rgb_base_colors {
    /// Values from "An RGB-to-spectrum conversion for reflectances"
    use super::ColorSpectrum10;
    use palette::*;

    static WHITE_SPECTRUM: ColorSpectrum10 = ColorSpectrum10 {
        spectrum: [
            1.0000,
            1.0000,
            0.9999,
            0.9993,
            0.9992,
            0.9998,
            1.0000,
            1.0000,
            1.0000,
            1.0000,
        ]
    };

    static CYAN_SPECTRUM: ColorSpectrum10 = ColorSpectrum10 {
        spectrum: [
            0.9710,
            0.9426,
            1.0007,
            1.0007,
            1.0007,
            1.0007,
            0.1564,
            0.0000,
            0.0000,
            0.0000,
        ]
    };

    static MAGENTA_SPECTRUM: ColorSpectrum10 = ColorSpectrum10 {
        spectrum: [
            1.0000,
            1.0000,
            0.9685,
            0.2229,
            0.0000,
            0.0458,
            0.8369,
            1.0000,
            1.0000,
            0.9959,
        ]
    };

    static YELLOW_SPECTRUM: ColorSpectrum10 = ColorSpectrum10 {
        spectrum: [
            0.0001,
            0.0000,
            0.1088,
            0.6651,
            1.0000,
            1.0000,
            0.9996,
            0.9586,
            0.9685,
            0.9840,
        ]
    };

    static RED_SPECTRUM: ColorSpectrum10 = ColorSpectrum10 {
        spectrum: [
            0.1012,
            0.0515,
            0.0000,
            0.0000,
            0.0000,
            0.0000,
            0.8325,
            1.0149,
            1.0149,
            1.0149,
        ]
    };

    static GREEN_SPECTRUM: ColorSpectrum10 = ColorSpectrum10 {
        spectrum: [
            0.0000,
            0.0000,
            0.0273,
            0.7937,
            1.0000,
            0.9418,
            0.1719,
            0.0000,
            0.0000,
            0.0025,
        ]
    };

    static BLUE_SPECTRUM: ColorSpectrum10 = ColorSpectrum10 {
        spectrum: [
            1.0000,
            1.0000,
            0.8916,
            0.3323,
            0.0000,
            0.0000,
            0.0003,
            0.0369,
            0.0483,
            0.0496,
        ]
    };

    pub fn rgb_to_spectrum(rgb: Rgb<f32>) -> ColorSpectrum10 {
        let red = rgb.red;
        let green = rgb.green;
        let blue = rgb.blue;
        let mut ret = ColorSpectrum10{ spectrum: [0.0; 10] };
        if red <= green && red <= blue {
            ret += red * WHITE_SPECTRUM;
            if green <= blue {
                ret += (green - red) * CYAN_SPECTRUM;
                ret += (blue - green) * BLUE_SPECTRUM;
            } else {
                ret += (blue - red) * CYAN_SPECTRUM;
                ret += (green - blue) * GREEN_SPECTRUM;
            }
        } else if green <= red && green <= blue {
            ret += green * WHITE_SPECTRUM;
            if red <= blue {
                ret += (blue - green) * MAGENTA_SPECTRUM;
                ret += (blue - red) * BLUE_SPECTRUM;
            } else {
                ret += (red - green) * MAGENTA_SPECTRUM;
                ret += (red - blue) * RED_SPECTRUM;
            }
        } else /* blue <= red && blue <= green */ {
            ret += blue * WHITE_SPECTRUM;
            if red <= green {
                ret += (green - blue) * YELLOW_SPECTRUM;
                ret += (green - red) * GREEN_SPECTRUM;
            } else {
                ret += (green - blue) * YELLOW_SPECTRUM;
                ret += (red - green) * RED_SPECTRUM;
            }
        }
        ret
    }
}

impl<C: IntoColor<f32>> HasReflectance for C {
    fn reflect(self, wl: f32) -> f32 {
        let color_rgb = self.into_rgb();
        let spectrum = rgb_base_colors::rgb_to_spectrum(color_rgb);
        spectrum.reflect(wl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::*;

    #[test]
    fn test_xyz_from_valid() {
        for i in 380..780 {
            let xyz = xyz_from_wavelength(i as f32);
            let xyz_clamped = xyz.clamp();
            assert_eq!(xyz, xyz_clamped, "Invalid xyz for wl={:}", i);
        }
    }

    #[test]
    fn test_reflect_wavelength() {
        for i in 380..780 {
            let xyz = xyz_from_wavelength(i as f32);
            let refl = xyz.reflect(i as f32);
            assert!(refl<=1.0, "Got non-correct reflection for {:}nm: {}", i, refl);
        }
    }

    #[test]
    fn test_match_color_rgb() {
        let white = Rgb::new(1.0, 1.0, 1.0);
        for i in 380..780 {
            let val = white.reflect(i as f32);
            assert!((val - 1.0).abs()<0.001
                    ,"White didn't match close to 1 for {:}nm, got instead: {:}"
                    , i, val
            );
        }
    }

    #[bench]
    fn bench_match_color_rgb(bench: &mut Bencher) {
        let white = black_box(Rgb::new(1.0, 1.0, 1.0));
        let wl = black_box(500.0);
        bench.iter(|| white.reflect(wl));
    }

    #[bench]
    fn bench_match_color_spectrum(bench: &mut Bencher) {
        let white = black_box(ColorSpectrum10{spectrum: [1.0; 10]});
        let wl = black_box(500.0);
        bench.iter(|| white.reflect(wl));
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
        let a = black_box(ColorSpectrum10{ spectrum: [0.2; 10] });
        let b = black_box(ColorSpectrum10{ spectrum: [0.4; 10] });
        bench.iter(|| {
            a+b
        });
    }
}
