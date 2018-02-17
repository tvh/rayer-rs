use palette::*;
use palette::white_point::E;
use std::fmt::Debug;

mod binned_spectrum;
mod cie_1931;
mod rgb_base_colors;

pub use self::cie_1931::xyz_from_wavelength;

pub trait HasReflectance: Debug + Send + Sync {
    fn reflect(&self, wl: f32) -> f32;
}


impl HasReflectance for Rgb<E, f32> where
{
    fn reflect(&self, wl: f32) -> f32 {
        let spectrum = rgb_base_colors::rgb_to_spectrum(*self);
        spectrum.reflect(wl)
    }
}


#[cfg(test)]
mod tests {
    use palette::white_point::WhitePoint;
    use super::*;
    use test::*;
    use quickcheck::{Arbitrary, Gen};

    #[derive(Clone, Debug)]
    struct TestRgb(Rgb<E, f32>);
    impl Arbitrary for TestRgb {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            TestRgb(Rgb::with_wp(g.next_f32(), g.next_f32(), g.next_f32()))
        }
    }

    #[test]
    fn test_xyz_from_valid() {
        let mut max_x = 0.0;
        let mut max_x_freq = 0;
        let mut max_y = 0.0;
        let mut max_y_freq = 0;
        let mut max_z = 0.0;
        let mut max_z_freq = 0;
        for i in 380..780 {
            let xyz = xyz_from_wavelength(i as f32);
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
        let white = E::get_xyz();
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
        let mut max_xyz = Xyz::with_wp(0.0, 0.0, 0.0);
        let mut max_freq = 0;
        for i in 380..780 {
            let xyz = xyz_from_wavelength(i as f32);
            let refl = xyz.into_rgb().reflect(i as f32);
            if refl>max_refl {
                max_refl=refl;
                max_freq=i;
                max_xyz=xyz;
            }
        }
        assert!(max_refl<=1.0, "Got non-correct reflection for wl={:}nm, refl={:}, xyz={:?}, rgb={:?}", max_freq, max_refl, max_xyz, max_xyz.into_rgb());
    }

    #[test]
    fn test_match_color_rgb_grey() {
        for &intensity in [0.0, 0.3, 0.5, 0.7, 1.0].iter() {
            let grey = Rgb::with_wp(intensity, intensity, intensity);
            for i in 380..780 {
                let val = grey.reflect(i as f32);
                assert!((val - intensity).abs()<0.001
                        ,"Reflectance is not like expected: wl={:}nm, intensity={:}, refl={:}"
                        , i, intensity, val
                );
            }
        }
    }

    #[quickcheck]
    fn reflected_intensity_seems_reasonable(rgb: TestRgb) -> () {
        let TestRgb(rgb) = rgb;
        let max_val = rgb.red.max(rgb.green).max(rgb.blue);
        let min_val = rgb.red.min(rgb.green).min(rgb.blue);
        for i in 380..780 {
            let val = rgb.reflect(i as f32);
            assert!(val>=min_val-0.01
                    ,"Reflectance is lower than the lowest contributer: wl={:}nm, rbg={:?}, refl={:}"
                    , i, rgb, val
            );
            assert!(val<=max_val+0.01
                    ,"Reflectance is above the highest contributor: wl={:}nm, rbg={:?}, refl={:}"
                    , i, rgb, val
            );
        }
    }

    #[bench]
    fn bench_match_color_rgb(bench: &mut Bencher) {
        let white = black_box(Rgb::with_wp(1.0, 1.0, 1.0));
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
