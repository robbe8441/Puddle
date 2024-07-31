use ash::vk;

pub mod compute;
pub mod graphics;

pub trait Pipeline {
    fn layout(&self) -> vk::PipelineLayout;
    fn bind_point(&self) -> vk::PipelineBindPoint;
    fn as_raw(&self) -> vk::Pipeline;
}
