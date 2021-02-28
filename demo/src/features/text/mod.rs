use crate::features::text::extract::TextExtractJob;
use rafx::framework::{VertexDataLayout, VertexDataSetLayout, ImageViewResource, ResourceArc};
use rafx::nodes::ExtractJob;
use rafx::nodes::RenderFeature;
use rafx::nodes::RenderFeatureIndex;
use std::convert::TryInto;

mod text_resource;
mod extract;
mod prepare;
mod write;

pub use text_resource::*;
use rafx::api::{RafxPrimitiveTopology, RafxBuffer};
use std::sync::Arc;

pub fn create_text_extract_job(
) -> Box<dyn ExtractJob> {
    Box::new(TextExtractJob::new())
}

pub type TextUniformBufferObject = shaders::text_vert::PerViewUboUniform;

/// Vertex format for vertices sent to the GPU
#[derive(Clone, Debug, Copy, Default)]
#[repr(C)]
pub struct TextVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

lazy_static::lazy_static! {
    pub static ref TEXT_VERTEX_LAYOUT : VertexDataSetLayout = {
        use rafx::api::RafxFormat;

        VertexDataLayout::build_vertex_layout(&TextVertex::default(), |builder, vertex| {
            builder.add_member(&vertex.position, "POSITION", RafxFormat::R32G32B32_SFLOAT);
            builder.add_member(&vertex.uv, "TEXCOORD", RafxFormat::R32G32_SFLOAT);
            builder.add_member(&vertex.color, "COLOR", RafxFormat::R32G32B32A32_SFLOAT);
        }).into_set(RafxPrimitiveTopology::TriangleList)
    };
}

rafx::declare_render_feature!(TextRenderFeature, TEXT_FEATURE_INDEX);

struct TextImageUpdate {
    // upload_buffer: RafxBuffer,
    // upload_rectangle: glyph_brush::Rectangle<u32>,
}

pub(self) struct ExtractedTextData {
    // If we need to update the image, these values will be set
    image_update: Option<TextImageUpdate>,

    // Either provides new vertex data or indicates to redraw previous vertex data
    vertex_data: Vec<TextVertex>,

    texture: Option<ResourceArc<ImageViewResource>>
}


