use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;

use anyhow::Result;
use ash::vk;

use bytemuck::offset_of;
use egui::epaint::{
    textures::TexturesDelta, ClippedPrimitive, ClippedShape, ImageData, ImageDelta, Primitive,
};
use egui::{Color32, Context, Rect, TextureId};

use crate::instances::descriptors::{BindingDescriptor, DescriptorSet, WriteDescriptorSet};
use crate::instances::graphics::{PipelineCreateInfo, PipelineGraphics, RenderPass};
use crate::instances::{
    BufferSlice, CommandBuffer, Device, Image, ImageView, ImageViewCreateInfo, Queue, Sampler,
    Subbuffer,
};
use crate::types::VertexInput;
mod shaders;

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(PartialEq)]
/// You must use this to avoid attempting to modify a texture that's still in use.
pub enum UpdateTexturesResult {
    /// No texture will be modified in this frame.
    Unchanged,
    /// A texture will be modified in this frame,
    /// and you must wait for the last frame to finish before submitting the next command buffer.
    Changed,
}

impl VertexInput for Vertex {
    fn desc() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                offset: offset_of!(Vertex, pos) as u32,
                format: ash::vk::Format::R32G32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                offset: offset_of!(Vertex, uv) as u32,
                format: ash::vk::Format::R32G32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                offset: offset_of!(Vertex, color) as u32,
                format: ash::vk::Format::R32G32B32A32_SFLOAT,
            },
        ]
    }
}

impl From<&egui::epaint::Vertex> for Vertex {
    fn from(v: &egui::epaint::Vertex) -> Self {
        let convert = {
            |c: Color32| {
                [
                    c.r() as f32 / 255.0,
                    c.g() as f32 / 255.0,
                    c.b() as f32 / 255.0,
                    c.a() as f32 / 255.0,
                ]
            }
        };

        Self {
            pos: [v.pos.x, v.pos.y],
            uv: [v.uv.x, v.uv.y],
            color: convert(v.color),
        }
    }
}

/// Contains everything needed to render the gui.
pub struct Painter {
    device: Arc<Device>,
    queue: Arc<Queue>,
    /// Graphics pipeline used to render the gui.
    pub pipeline: Arc<PipelineGraphics>,
    /// Texture sampler used to render the gui.
    pub sampler: Arc<Sampler>,
    images: HashMap<egui::TextureId, Arc<Image>>,
    texture_sets: HashMap<egui::TextureId, Arc<DescriptorSet>>,
    texture_free_queue: Vec<egui::TextureId>,
}

impl Painter {
    /// Pass in the vulkano [`Device`], [`Queue`] and [`Subpass`]
    /// that you want to use to render the gui.
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        renderpass: Arc<RenderPass>,
    ) -> Result<Self> {
        let pipeline = create_pipeline(device.clone(), renderpass.clone())?;
        let sampler = create_sampler(device.clone())?;
        Ok(Self {
            device,
            queue,
            pipeline,
            sampler,
            images: Default::default(),
            texture_sets: Default::default(),
            texture_free_queue: Vec::new(),
        })
    }

    fn write_image_delta(
        &mut self,
        image: Arc<Image>,
        delta: &ImageDelta,
        command_buffer: &mut CommandBuffer,
    ) -> Result<()> {
        let image_data = match &delta.image {
            ImageData::Color(image) => image
                .pixels
                .iter()
                .flat_map(|c| c.to_array())
                .collect::<Vec<_>>(),
            ImageData::Font(image) => image
                .srgba_pixels(1.0)
                .flat_map(|c| c.to_array())
                .collect::<Vec<_>>(),
        };

        let img_buffer = Subbuffer::from_data(
            self.device.clone(),
            vk::BufferCreateInfo {
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                ..Default::default()
            },
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            &image_data,
        )?;

        let size = [delta.image.width() as u32, delta.image.height() as u32, 1];
        let offset = match delta.pos {
            None => [0, 0, 0],
            Some(pos) => [pos[0] as u32, pos[1] as u32, 0],
        };

        command_buffer.copy_buffer_to_image_dimensions(img_buffer.clone(), image.clone(), offset, size);
        // builder.copy_buffer_to_image_dimensions(img_buffer, image, offset, size, 0, 1, 0)?;
        Ok(())
    }

    /// Uploads all newly created and modified textures to the GPU.
    /// Has to be called before entering the first render pass.  
    /// If the return value is [`UpdateTexturesResult::Changed`],
    /// a texture will be changed in this frame and you need to wait for the last frame to finish
    /// before submitting the command buffer for this frame.
    pub fn update_textures(
        &mut self,
        textures_delta: TexturesDelta,
        command_buffer: &mut CommandBuffer,
    ) -> Result<UpdateTexturesResult>
    {
        for texture_id in textures_delta.free {
            self.texture_free_queue.push(texture_id);
        }

        let mut result = UpdateTexturesResult::Unchanged;
        use crate::instances::Pipeline;

        for (texture_id, delta) in &textures_delta.set {
            let image = if delta.is_whole() {
                let image = create_image(self.device.clone(), &delta.image)?;

                let bindings = [BindingDescriptor {
                    binding: 0,
                    count: 1,
                    shader_stage: vk::ShaderStageFlags::FRAGMENT,
                    ty: crate::instances::descriptors::DescriptorType::StorageImage,
                }];

                let image_view_info = ImageViewCreateInfo {
                    view_type: vk::ImageViewType::TYPE_2D,
                    format: image.create_info().format,
                    components: vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    },
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        level_count: 1,
                        layer_count: 1,
                        ..Default::default()
                    },
                };

                let view = ImageView::new(image.clone(), &image_view_info)?;

                let set = DescriptorSet::new(self.device.clone(), &bindings)?;

                let writes = [WriteDescriptorSet::ImageViews(0, vec![view])];

                set.write(&writes);

                self.texture_sets.insert(*texture_id, set);
                self.images.insert(*texture_id, image.clone());
                image
            } else {
                result = UpdateTexturesResult::Changed; //modifying an existing image that might be in use
                self.images[texture_id].clone()
            };
            self.write_image_delta(image, delta, command_buffer)?;
        }

        Ok(result)
    }

    /// Free textures freed by egui, *after* drawing
    fn free_textures(&mut self) {
        for texture_id in &self.texture_free_queue {
            self.texture_sets.remove(texture_id);
            self.images.remove(texture_id);
        }

        self.texture_free_queue.clear();
    }

    /// Advances to the next rendering subpass and uses the [`ClippedShape`]s from [`egui::FullOutput`] to draw the gui.
    pub fn draw(
        &mut self,
        command_buffer: &mut CommandBuffer,
        window_size_points: [f32; 2],
        egui_ctx: &Context,
        clipped_shapes: Vec<ClippedShape>,
    ) -> Result<()> {
        command_buffer.bind_pipeline(self.pipeline.clone());

        let clipped_primitives: Vec<ClippedPrimitive> = egui_ctx.tessellate(clipped_shapes);
        let num_meshes = clipped_primitives.len();

        let mut verts = Vec::<Vertex>::with_capacity(num_meshes * 4);
        let mut indices = Vec::<u32>::with_capacity(num_meshes * 6);
        let mut clips = Vec::<Rect>::with_capacity(num_meshes);
        let mut texture_ids = Vec::<TextureId>::with_capacity(num_meshes);
        let mut offsets = Vec::<(usize, usize)>::with_capacity(num_meshes);

        for cm in clipped_primitives.iter() {
            let clip = cm.clip_rect;
            let mesh = match &cm.primitive {
                Primitive::Mesh(mesh) => mesh,
                Primitive::Callback(_) => {
                    continue; // callbacks not supported at the moment
                }
            };

            // Skip empty meshes
            if mesh.vertices.len() == 0 || mesh.indices.len() == 0 {
                continue;
            }

            offsets.push((verts.len(), indices.len()));
            texture_ids.push(mesh.texture_id);

            for v in mesh.vertices.iter() {
                verts.push(v.into());
            }

            for i in mesh.indices.iter() {
                indices.push(*i);
            }

            clips.push(clip);
        }
        offsets.push((verts.len(), indices.len()));

        // Return if there's nothing to render
        if clips.len() == 0 {
            return Ok(());
        }

        let (vertex_buf, index_buf) = self.create_buffers((verts, indices))?;
        for (idx, clip) in clips.iter().enumerate() {
            let mut scissors = Vec::with_capacity(1);
            let o = clip.min;
            let (w, h) = (clip.width() as u32, clip.height() as u32);
            scissors.push(vk::Rect2D {
                offset: vk::Offset2D {
                    x: o.x as i32,
                    y: o.y as i32,
                },
                extent: vk::Extent2D {
                    width: w,
                    height: h,
                },
            });
            command_buffer.set_scissor(0, &scissors);

            let offset = offsets[idx];
            let end = offsets[idx + 1];

            let vb_slice = BufferSlice::new(vertex_buf.clone(), offset.0 as u64, end.0 as u64);
            let ib_slice = BufferSlice::new(index_buf.clone(), offset.1 as u64, end.1 as u64);

            let texture_set = self.texture_sets.get(&texture_ids[idx]);
            if texture_set.is_none() {
                continue; //skip if we don't have a texture
            }

            command_buffer.bind_vertex_buffers(0, &[vb_slice.clone()], &[0]);
            command_buffer.bind_index_buffer(ib_slice.clone(), 0, vk::IndexType::UINT32);
            command_buffer.bind_descriptor_set(
                texture_set.unwrap().clone(),
                0,
                self.pipeline.clone(),
                &[],
            );
            use crate::instances::Pipeline;
            command_buffer.push_constants(
                self.pipeline.layout(),
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::cast_slice(&window_size_points),
            );
            command_buffer.draw_indexed(end.1 as u32, 1, 0, 0, 0);
        }
        self.free_textures();
        Ok(())
    }

    /// Create vulkano CpuAccessibleBuffer objects for the vertices and indices
    fn create_buffers(
        &self,
        triangles: (Vec<Vertex>, Vec<u32>),
    ) -> Result<(Arc<Subbuffer<Vertex>>, Arc<Subbuffer<u32>>)> {
        let vertex_buffer = Subbuffer::from_data(
            self.device.clone(),
            vk::BufferCreateInfo {
                usage: vk::BufferUsageFlags::VERTEX_BUFFER,
                ..Default::default()
            },
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            &triangles.0,
        )?;

        let index_buffer = Subbuffer::from_data(
            self.device.clone(),
            vk::BufferCreateInfo {
                usage: vk::BufferUsageFlags::INDEX_BUFFER,
                ..Default::default()
            },
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            &triangles.1,
        )?;

        Ok((vertex_buffer, index_buffer))
    }
}

/// Create a graphics pipeline with the shaders and settings necessary to render egui output
fn create_pipeline(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
) -> Result<Arc<PipelineGraphics>> {
    let (vs, fs) = shaders::load(device.clone());

    let bindings = [BindingDescriptor {
        binding: 0,
        count: 1,
        shader_stage: vk::ShaderStageFlags::FRAGMENT,
        ty: crate::instances::descriptors::DescriptorType::StorageImage,
    }];

    let descriptors = DescriptorSet::new(device.clone(), &bindings)?;

    let pipeline_info = PipelineCreateInfo {
        vertex_shader: vs,
        fragment_shader: fs,
        descriptor_layouts: vec![descriptors.layout()],
        cull_mode: crate::instances::graphics::CullMode::None,
        render_pass,
        device,
        vertex_input: Vertex::default(),
    };

    let pipeline = PipelineGraphics::new(pipeline_info)?;
    Ok(pipeline)
}

/// Create a texture sampler for the textures used by egui
fn create_sampler(device: Arc<Device>) -> Result<Arc<Sampler>> {
    let sampler_info = vk::SamplerCreateInfo {
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        address_mode_u: vk::SamplerAddressMode::MIRRORED_REPEAT,
        address_mode_v: vk::SamplerAddressMode::MIRRORED_REPEAT,
        address_mode_w: vk::SamplerAddressMode::MIRRORED_REPEAT,
        max_anisotropy: 1.0,
        border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
        compare_op: vk::CompareOp::NEVER,
        ..Default::default()
    };

    Sampler::new(device.clone(), &sampler_info)
}

/// Create a Vulkano image for the given egui texture
fn create_image(device: Arc<Device>, texture: &ImageData) -> Result<Arc<crate::instances::Image>> {
    let image_create_info = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_2D,
        format: vk::Format::R8G8B8A8_SRGB,
        extent: vk::Extent3D {
            width: texture.width() as u32,
            height: texture.height() as u32,
            depth: 1,
        },
        mip_levels: 1,
        array_layers: 1,
        samples: vk::SampleCountFlags::TYPE_1,
        tiling: vk::ImageTiling::OPTIMAL,
        usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let image = Image::new(device.clone(), image_create_info)?;

    Ok(image)
}
