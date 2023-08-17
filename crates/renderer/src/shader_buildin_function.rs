use math::{Vec2, Vec4};

use crate::{Sampler, Texture, TextureFormat};

// 纹理采样
pub fn texture_sample(texture: &Texture, _sampler: Sampler, mut coords: Vec2) -> Vec4 {
    if coords.x > 1.0 {
        coords.x -= coords.x.floor();
    }
    if coords.y > 1.0 {
        coords.y -= coords.y.floor();
    }
    let x = (coords.x * (texture.width - 1) as f32) as usize;
    let y = (coords.y * (texture.height - 1) as f32) as usize;

    match texture.format {
        TextureFormat::R8Unorm => {
            let index = (y * texture.width as usize) + x;
            [texture.data[index] as f32 / 255.; 4].into()
        }
        TextureFormat::Rgb8Unorm => {
            let index = 3 * ((y * texture.width as usize) + x);
            [
                texture.data[index] as f32 / 255.,
                texture.data[index + 1] as f32 / 255.,
                texture.data[index + 2] as f32 / 255.,
                1.,
            ]
            .into()
        }
        TextureFormat::Rgba8Unorm => {
            let index = 4 * ((y * texture.width as usize) + x);
            [
                texture.data[index] as f32 / 255.,
                texture.data[index + 1] as f32 / 255.,
                texture.data[index + 2] as f32 / 255.,
                texture.data[index + 3] as f32 / 255.,
            ]
            .into()
        }
    }
}
