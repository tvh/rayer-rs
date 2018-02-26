use euclid::*;
use palette;
use color::*;
use std::fmt::Debug;
use image::*;
use std::sync::Arc;
use num_traits::ToPrimitive;
use palette::white_point::E;

pub trait Texture: Debug + Send + Sync {
    fn value(&self, uv: Vector2D<f32>, wl: f32) -> f32;
}

impl<C: HasReflectance> Texture for C {
    fn value(&self, _uv: Vector2D<f32>, wl: f32) -> f32 {
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

impl Texture for ImageTexture {
    fn value(&self, uv: Vector2D<f32>, wl: f32) -> f32 {
        let nx = self.image.width();
        let ny = self.image.height();
        let i: isize = (uv.x*(nx as f32)).to_isize().unwrap();
        let j: isize = ((1.0 - uv.y)*(ny as f32)-0.001).to_isize().unwrap();
        let i: u32 = i.max(0).min(nx as isize).to_u32().unwrap();
        let j: u32 = j.max(0).min(ny as isize).to_u32().unwrap();
        let Rgb{ data: [r,g,b] } = self.image[(i, j)];
        let rgbf: palette::Rgb<E, f32> = palette::pixel::Srgb::with_wp(
            r as f32/255.0,
            g as f32/255.0,
            b as f32/255.0,
        ).into();
        rgbf.reflect(wl)
    }
}
