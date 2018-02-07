use palette::*;

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
