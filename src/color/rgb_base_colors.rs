use palette::*;
use palette::white_point::D65;
use super::binned_spectrum::{BinnedSpectrum, BinData};

#[derive(Debug)]
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

pub fn rgb_to_spectrum(rgb: Rgb<D65, f32>) -> ColorSpectrum10 {
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
            ret += (red - green) * MAGENTA_SPECTRUM;
            ret += (blue - red) * BLUE_SPECTRUM;
        } else {
            ret += (blue - green) * MAGENTA_SPECTRUM;
            ret += (red - blue) * RED_SPECTRUM;
        }
    } else /* blue <= red && blue <= green */ {
        ret += blue * WHITE_SPECTRUM;
        if red <= green {
            ret += (red - blue) * YELLOW_SPECTRUM;
            ret += (green - red) * GREEN_SPECTRUM;
        } else {
            ret += (green - blue) * YELLOW_SPECTRUM;
            ret += (red - green) * RED_SPECTRUM;
        }
    }
    ret
}
