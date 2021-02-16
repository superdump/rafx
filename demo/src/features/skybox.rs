// Not a full-on proper feature, but we'll put some skybox-specific stuff here anyways

use rafx::api::RafxPrimitiveTopology;
use rafx::framework::{
    VertexDataLayout, VertexDataSetLayout,
};

/// Vertex format for vertices sent to the GPU
#[derive(Clone, Debug, Copy, Default)]
#[repr(C)]
pub struct SkyboxVertex(pub [f32; 3]);

lazy_static::lazy_static! {
    pub static ref SKYBOX_VERTEX_LAYOUT : VertexDataSetLayout = {
        use rafx::api::RafxFormat;

        VertexDataLayout::build_vertex_layout(&SkyboxVertex::default(), |builder, vertex| {
            builder.add_member(&vertex.0, "POSITION", RafxFormat::R32G32B32_SFLOAT);
        }).into_set(RafxPrimitiveTopology::TriangleList)
    };
}


pub const SKYBOX_CUBE_VERTEX_BUFFER_DATA : [SkyboxVertex; 8] = [
    SkyboxVertex([-1.0, -1.0, -1.0]),
    SkyboxVertex([1.0, -1.0, -1.0]),
    SkyboxVertex([1.0, 1.0, -1.0]),
    SkyboxVertex([-1.0, 1.0, -1.0]),
    SkyboxVertex([-1.0, -1.0, 1.0]),
    SkyboxVertex([1.0, -1.0, 1.0]),
    SkyboxVertex([1.0, 1.0, 1.0]),
    SkyboxVertex([-1.0, 1.0, 1.0])
];

pub const SKYBOX_CUBE_INDEX_BUFFER_DATA : [u16; 36] = [
    0, 1, 3,
    3, 1, 2,
    1, 5, 2,
    2, 5, 6,
    5, 4, 6,
    6, 4, 7,
    4, 0, 7,
    7, 0, 3,
    3, 2, 7,
    7, 2, 6,
    4, 5, 0,
    0, 5, 1,
];
