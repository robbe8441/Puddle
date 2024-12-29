use ash::vk;

use super::{node::Node, resource::ResourceDescriptor};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub struct RenderGraphBuilder {
    nodes: Vec<Box<dyn Node>>,
    render_attachments_desciptors: HashMap<TypeId, vk::AttachmentDescription>,
}

impl RenderGraphBuilder {
    pub fn register_render_attachment<T: Any>(&mut self, desc: vk::AttachmentDescription) {
        self.render_attachments_desciptors
            .insert(TypeId::of::<T>(), desc);
    }

    pub fn register_node(&mut self, node: impl Node + 'static) {
        self.nodes.push(Box::new(node));
    }
}

pub struct NodeBuildContext {
    resources: HashMap<TypeId, ResourceDescriptor>,

    render_attachments: HashMap<TypeId, vk::ImageLayout>,
}

impl NodeBuildContext {
    pub fn request_resource<T: Any>(&mut self, desc: ResourceDescriptor) {
        self.resources.insert(TypeId::of::<T>(), desc);
    }

    pub fn request_render_attachment<T: Any>(&mut self, layout: vk::ImageLayout) {
        self.render_attachments.insert(TypeId::of::<T>(), layout);
    }
}

pub struct GraphContext {}
