use euclid::*;
use palette;
use color::*;
use types::CoordinateBase;
use std::fmt::Debug;
use image::*;
use std::sync::Arc;
use num_traits::ToPrimitive;

pub trait Texture<T: CoordinateBase>: Debug + Send + Sync {
    fn value(&self, uv: Vector2D<T>, wl: f32) -> f32;
}

impl<C: HasReflectance, T: CoordinateBase> Texture<T> for C {
    fn value(&self, _uv: Vector2D<T>, wl: f32) -> f32 {
        self.reflect(wl)
    }
}

#[derive(Debug, Clone)]
pub struct ImageTexture {
    image: Arc<RgbImage>,
}

impl ImageTexture {
    pub fn new(image: &Arc<RgbImage>) -> ImageTexture {
        ImageTexture { image: image.clone() }
    }
}

impl<T: CoordinateBase> Texture<T> for ImageTexture {
    fn value(&self, uv: Vector2D<T>, wl: f32) -> f32 {
        let nx = self.image.width();
        let ny = self.image.height();
        let i: isize = (uv.x*From::from(nx as f32)).to_isize().unwrap();
        let j: isize = ((T::one() - uv.y)*From::from(ny as f32)-From::from(0.001)).to_isize().unwrap();
        let i: u32 = i.max(0).min(nx as isize).to_u32().unwrap();
        let j: u32 = j.max(0).min(ny as isize).to_u32().unwrap();
        let Rgb{ data: [r,g,b] } = self.image[(i, j)];
        let rgbf = palette::Rgb::new(
            r as f32/255.0,
            g as f32/255.0,
            b as f32/255.0,
        );
        rgbf.reflect(wl)
    }
}
