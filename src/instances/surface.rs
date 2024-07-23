use super::Instance;
use anyhow::Result;
use std::{os::raw::c_char, sync::Arc};

use ash::{
    ext::metal_surface,
    khr::{android_surface, surface, wayland_surface, win32_surface, xcb_surface, xlib_surface},
    prelude::*,
    vk,
};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

pub struct Surface {
    intern: vk::SurfaceKHR,
    loader: ash::khr::surface::Instance,
}

impl Surface {
    pub unsafe fn new(
        instance: Arc<Instance>,
        window: impl HasWindowHandle + HasDisplayHandle,
    ) -> Result<Self> {
        let instance_raw = instance.as_raw();
        let entry = instance.entry();

        let display_handle = window.display_handle().unwrap().as_raw();
        let window_handle = window.window_handle().unwrap().as_raw();

        let surface = match (display_handle, window_handle) {
            (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(window)) => {
                let surface_desc = vk::Win32SurfaceCreateInfoKHR::default()
                    .hwnd(window.hwnd.get())
                    .hinstance(
                        window
                            .hinstance
                            .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?
                            .get(),
                    );
                let surface_fn = win32_surface::Instance::new(&entry, &instance_raw);
                surface_fn.create_win32_surface(&surface_desc, None)
            }

            (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(window)) => {
                let surface_desc = vk::WaylandSurfaceCreateInfoKHR::default()
                    .display(display.display.as_ptr())
                    .surface(window.surface.as_ptr());
                let surface_fn = wayland_surface::Instance::new(&entry, &instance_raw);
                surface_fn.create_wayland_surface(&surface_desc, None)
            }

            (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(window)) => {
                let surface_desc = vk::XlibSurfaceCreateInfoKHR::default()
                    .dpy(
                        display
                            .display
                            .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?
                            .as_ptr(),
                    )
                    .window(window.window);
                let surface_fn = xlib_surface::Instance::new(&entry, &instance_raw);
                surface_fn.create_xlib_surface(&surface_desc, None)
            }

            (RawDisplayHandle::Xcb(display), RawWindowHandle::Xcb(window)) => {
                let surface_desc = vk::XcbSurfaceCreateInfoKHR::default()
                    .connection(
                        display
                            .connection
                            .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?
                            .as_ptr(),
                    )
                    .window(window.window.get());
                let surface_fn = xcb_surface::Instance::new(&entry, &instance_raw);
                surface_fn.create_xcb_surface(&surface_desc, None)
            }

            (RawDisplayHandle::Android(_), RawWindowHandle::AndroidNdk(window)) => {
                let surface_desc = vk::AndroidSurfaceCreateInfoKHR::default()
                    .window(window.a_native_window.as_ptr());
                let surface_fn = android_surface::Instance::new(&entry, &instance_raw);
                surface_fn.create_android_surface(&surface_desc, None)
            }

            #[cfg(target_os = "macos")]
            (RawDisplayHandle::AppKit(_), RawWindowHandle::AppKit(window)) => {
                use raw_window_metal::{appkit, Layer};

                let layer = match appkit::metal_layer_from_handle(window) {
                    Layer::Existing(layer) | Layer::Allocated(layer) => layer.cast(),
                };

                let surface_desc = vk::MetalSurfaceCreateInfoEXT::default().layer(&*layer);
                let surface_fn = metal_surface::Instance::new(&entry, &instance_raw);
                surface_fn.create_metal_surface(&surface_desc, None)
            }

            #[cfg(target_os = "ios")]
            (RawDisplayHandle::UiKit(_), RawWindowHandle::UiKit(window)) => {
                use raw_window_metal::{uikit, Layer};

                let layer = match uikit::metal_layer_from_handle(window) {
                    Layer::Existing(layer) | Layer::Allocated(layer) => layer.cast(),
                };

                let surface_desc = vk::MetalSurfaceCreateInfoEXT::default().layer(&*layer);
                let surface_fn = metal_surface::Instance::new(&entry, &instance_raw);
                surface_fn.create_metal_surface(&surface_desc, None)
            }

            _ => Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT),
        }?;

        let loader = surface::Instance::new(&entry, &instance_raw);

        Ok(Self { intern: surface, loader })
    }

    pub fn enumerate_required_extensions(
        display_handle: RawDisplayHandle,
    ) -> VkResult<&'static [*const c_char]> {
        let extensions = match display_handle {
            RawDisplayHandle::Windows(_) => {
                const WINDOWS_EXTS: [*const c_char; 2] =
                    [surface::NAME.as_ptr(), win32_surface::NAME.as_ptr()];
                &WINDOWS_EXTS
            }

            RawDisplayHandle::Wayland(_) => {
                const WAYLAND_EXTS: [*const c_char; 2] =
                    [surface::NAME.as_ptr(), wayland_surface::NAME.as_ptr()];
                &WAYLAND_EXTS
            }

            RawDisplayHandle::Xlib(_) => {
                const XLIB_EXTS: [*const c_char; 2] =
                    [surface::NAME.as_ptr(), xlib_surface::NAME.as_ptr()];
                &XLIB_EXTS
            }

            RawDisplayHandle::Xcb(_) => {
                const XCB_EXTS: [*const c_char; 2] =
                    [surface::NAME.as_ptr(), xcb_surface::NAME.as_ptr()];
                &XCB_EXTS
            }

            RawDisplayHandle::Android(_) => {
                const ANDROID_EXTS: [*const c_char; 2] =
                    [surface::NAME.as_ptr(), android_surface::NAME.as_ptr()];
                &ANDROID_EXTS
            }

            RawDisplayHandle::AppKit(_) | RawDisplayHandle::UiKit(_) => {
                const METAL_EXTS: [*const c_char; 2] =
                    [surface::NAME.as_ptr(), metal_surface::NAME.as_ptr()];
                &METAL_EXTS
            }

            _ => return Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT),
        };

        Ok(extensions)
    }

    pub fn loader(&self) -> ash::khr::surface::Instance {
        self.loader.clone()
    }

    pub fn as_raw(&self) -> vk::SurfaceKHR {
        self.intern.clone()
    }
}
