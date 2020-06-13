pub mod renderpass;

pub mod imgui_support;

mod game_renderer;
pub use game_renderer::GameRenderer;
pub use game_renderer::GameRendererWithContext;

pub mod time;

pub mod asset_resource;
pub mod asset_storage;
pub mod pipeline;
pub mod image_utils;
//pub mod upload;
pub mod resource_managers;
pub mod push_buffer;
pub mod pipeline_description;

use legion::prelude::*;
use glam::Vec3;
use renderer_base::visibility::DynamicAabbVisibilityNodeHandle;

//
// Everything below this point is only being used by the api_design example for prototyping purposes
//
pub mod features;
use features::sprite::SpriteRenderNodeHandle;
use crate::features::demo::DemoRenderNodeHandle;
use renderer_shell_vulkan::VkResourceDropSink;

pub mod phases;

pub struct CommandWriterContext {

}

#[derive(Copy, Clone)]
pub struct PositionComponent {
    pub position: Vec3,
}

#[derive(Clone)]
pub struct SpriteComponent {
    pub sprite_handle: SpriteRenderNodeHandle,
    pub visibility_handle: DynamicAabbVisibilityNodeHandle,
    pub alpha: f32,
}

pub struct ExtractSource {
    world: &'static World,
    resources: &'static Resources,
}

impl ExtractSource {
    pub fn new<'a>(
        world: &'a World,
        resources: &'a Resources,
    ) -> Self {
        unsafe {
            ExtractSource {
                world: force_to_static_lifetime(world),
                resources: force_to_static_lifetime(resources),
            }
        }
    }
}

unsafe fn force_to_static_lifetime<T>(value: &T) -> &'static T {
    std::mem::transmute(value)
}


//
// Just for demonstration of minimal API
//
pub struct DemoCommandWriterContext;

#[derive(Clone)]
pub struct DemoComponent {
    pub render_node_handle: DemoRenderNodeHandle,
    pub visibility_handle: DynamicAabbVisibilityNodeHandle,
    pub alpha: f32,
}