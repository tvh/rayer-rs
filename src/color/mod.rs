use palette::*;
use palette::white_point::D65;

mod binned_spectrum;
mod cie_1931;

pub use self::cie_1931::xyz_from_wavelength;

pub trait HasReflectance {
    fn reflect(self, wl: f32) -> f32;
}

mod rgb_base_colors {
    use palette::*;
    use palette::white_point::WhitePoint;
    use super::binned_spectrum::{BinnedSpectrum, BinData};

    pub struct Bin10;
    impl BinData for Bin10 {
        type Spectrum = [f32; 10];
        const WL_0: f32 = 380.0;
        const BIN_WIDTH: f32 = 34.0;
    }

    /// Values from "An RGB-to-spectrum conversion for reflectances"
    type ColorSpectrum10 = BinnedSpectrum<Bin10>;

    static WHITE_SPECTRUM: ColorSpectrum10 = ColorSpectrum10::new([
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
    ]);

    static CYAN_SPECTRUM: ColorSpectrum10 = ColorSpectrum10::new([
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
    ]);

    static MAGENTA_SPECTRUM: ColorSpectrum10 = ColorSpectrum10::new([
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
    ]);

    static YELLOW_SPECTRUM: ColorSpectrum10 = ColorSpectrum10::new([
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
    ]);

    static RED_SPECTRUM: ColorSpectrum10 = ColorSpectrum10::new([
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
    ]);

    static GREEN_SPECTRUM: ColorSpectrum10 = ColorSpectrum10::new([
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
    ]);

    static BLUE_SPECTRUM: ColorSpectrum10 = ColorSpectrum10::new([
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
    ]);

    pub fn rgb_to_spectrum<Wp: WhitePoint<f32>>(rgb: Rgb<Wp, f32>) -> ColorSpectrum10 {
        let red = rgb.red;
        let green = rgb.green;
        let blue = rgb.blue;
        let mut ret = ColorSpectrum10::new([0.0; 10]);
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

impl<C> HasReflectance for C where
    C: IntoColor<D65, f32>,
{
    fn reflect(self, wl: f32) -> f32 {
        let color_rgb = self.into_rgb();
        let spectrum = rgb_base_colors::rgb_to_spectrum(color_rgb);
        spectrum.reflect(wl)
    }
}

#[cfg(test)]
mod tests {
    use palette::white_point::WhitePoint;
    use super::*;
    use test::*;

    #[test]
    fn test_xyz_from_valid() {
        let mut max_x = 0.0;
        let mut max_x_freq = 0;
        let mut max_y = 0.0;
        let mut max_y_freq = 0;
        let mut max_z = 0.0;
        let mut max_z_freq = 0;
        for i in 380..780 {
            let xyz: Xyz<D65, f32> = xyz_from_wavelength(i as f32);
            if xyz.x>max_x {
                max_x = xyz.x;
                max_x_freq = i;
            }
            if xyz.y>max_y {
                max_y = xyz.y;
                max_y_freq = i;
            }
            if xyz.z>max_z {
                max_z = xyz.z;
                max_z_freq = i;
            }
        }
        let white = D65::get_xyz();
        let mut errors = String::new();
        if max_x>white.x {
            errors.push_str(&format!("Invalid x for wl={:}nm, x={:}\n", max_x_freq, max_x));
        }
        if max_y>white.y {
            errors.push_str(&format!("Invalid y for wl={:}nm, y={:}\n", max_y_freq, max_y));
        }
        if max_z>white.z {
            errors.push_str(&format!("Invalid z for wl={:}nm, z={:}\n", max_z_freq, max_z));
        }
        assert!(errors.len()==0, errors);
    }

    #[test]
    fn test_reflect_wavelength() {
        let mut max_refl = 0.0;
        let mut max_xyz = Xyz::new(0.0, 0.0, 0.0);
        let mut max_freq = 0;
        for i in 380..780 {
            let xyz = xyz_from_wavelength(i as f32);
            let refl = xyz.reflect(i as f32);
            if refl>max_refl {
                max_refl=refl;
                max_freq=i;
                max_xyz=xyz;
            }
        }
        assert!(max_refl<=1.0, "Got non-correct reflection for wl={:}nm, refl={:}, xyz={:?}, rgb={:?}", max_freq, max_refl, max_xyz, max_xyz.into_rgb());
    }

    #[test]
    fn test_match_color_rgb() {
        for &intensity in [0.0, 0.3, 0.5, 0.7, 1.0].iter() {
            let grey = Rgb::new(intensity, intensity, intensity);
            for i in 380..780 {
                let val = grey.reflect(i as f32);
                assert!((val - intensity).abs()<0.001
                        ,"Relectance is not like expected: wl={:}nm, intensity={:}, refl={:}"
                        , i, intensity, val
                );
            }
        }
    }

    #[bench]
    fn bench_match_color_rgb(bench: &mut Bencher) {
        let white = black_box(Rgb::new(1.0, 1.0, 1.0));
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
}
