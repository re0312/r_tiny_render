pub trait BindingType {}
pub struct EmptyBindingType;
impl BindingType for EmptyBindingType {}
pub struct Uniform {}
impl BindingType for Uniform {}
pub struct Storage {}
impl BindingType for Storage {}
pub struct Sampler {}
impl BindingType for Sampler {}
pub struct Texture {}
impl BindingType for Texture {}

pub type BindGroup = Vec<Box<dyn BindingType>>;

impl Clone for Box<dyn BindingType> {
    fn clone(&self) -> Self {
        return Box::new(EmptyBindingType) as Box<dyn BindingType>;
    }
}
