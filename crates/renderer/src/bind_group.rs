#[derive(Debug, Clone)]
pub struct Uniform {}
#[derive(Debug, Clone)]
pub struct Storage {}
#[derive(Debug, Clone)]
pub struct Sampler {}
#[derive(Debug, Clone)]
pub struct Texture {


}

#[derive(Debug, Clone)]
pub enum BindType {
    // u32存储uniform对应的索引
    Uniform(u32),
    Sampler(Sampler),
    Texture(Texture),
}

pub type BindGroup = Vec<BindType>;
