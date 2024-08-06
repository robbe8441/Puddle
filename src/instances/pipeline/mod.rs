use ash::vk;

pub mod compute;
pub mod graphics_old;
pub use graphics_old as graphics;

pub trait Pipeline {
    fn layout(&self) -> vk::PipelineLayout;
    fn bind_point(&self) -> vk::PipelineBindPoint;
    fn as_raw(&self) -> vk::Pipeline;
}
