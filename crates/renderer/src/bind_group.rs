use std::default;

use crate::TextureFormat;

#[derive(Debug, Clone)]
pub struct Uniform {}
#[derive(Debug, Clone)]
pub struct Storage {}
#[derive(Debug, Clone, Default)]
pub struct Sampler {}
#[derive(Debug, Clone, Default)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub enum BindType {
    // u32存储uniform对应的索引
    Uniform(Vec<u8>),
    Sampler(Sampler),
    Texture(Texture),
    #[default]
    Empty,
}
impl From<BindType> for Texture {
    fn from(value: BindType) -> Self {
        let val = value.clone();
        match val {
            BindType::Texture(v) => v,
            _ => Texture::default(),
        }
    }
}
impl From<BindType> for Sampler {
    fn from(value: BindType) -> Self {
        match value {
            BindType::Sampler(v) => v,
            _ => Sampler::default(),
        }
    }
}

pub type BindGroup = Vec<BindType>;
