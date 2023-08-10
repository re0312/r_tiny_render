pub trait BindingType {}
pub struct Uniform {}
pub struct Storage {}
pub struct Sampler {}
pub struct Texture {}

pub type BindGroup = Vec<Box<dyn BindingType>>;
