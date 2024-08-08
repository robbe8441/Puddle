use anyhow::Result;
use std::sync::Arc;

use ash::vk;

pub struct ImageView {
    intern: vk::ImageView,
    image: Arc<super::Image>,
}

pub struct ImageViewCreateInfo {
    pub view_type: vk::ImageViewType,
    pub format: vk::Format,
    pub components: vk::ComponentMapping,
    pub subresource_range: vk::ImageSubresourceRange,
}

impl ImageView {
    pub fn new(image: Arc<super::Image>, create_info: &ImageViewCreateInfo) -> Result<Arc<Self>> {
        let create_info = vk::ImageViewCreateInfo {
            view_type: create_info.view_type,
            format: create_info.format,
            components: create_info.components,
            subresource_range: create_info.subresource_range,
            image: image.as_raw(),
            ..Default::default()
        };

        let image_view = unsafe {
            image
                .device()
                .as_raw()
                .create_image_view(&create_info, None)
        }?;

        Ok(Self {
            intern: image_view,
            image,
        }
        .into())
    }

    pub fn image(&self) -> Arc<super::Image> {
        self.image.clone()
    }

    pub fn as_raw(&self) -> vk::ImageView {
        self.intern
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.image
                .device()
                .as_raw()
                .destroy_image_view(self.intern, None)
        };
    }
}
