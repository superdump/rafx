use crate::features::text::prepare::TextPrepareJobImpl;
use crate::features::text::{TextRenderFeature, ExtractedTextData, TextResource, TextVertex, TextImageUpdate};
use crate::game_renderer::GameRendererStaticResources;
use rafx::nodes::{ExtractJob, FramePacket, PrepareJob, RenderFeature, RenderFeatureIndex, RenderView, RenderJobExtractContext};
use glyph_brush::{BrushError, BrushAction, GlyphVertex};
use rafx::api::{RafxBufferDef, RafxResourceType, RafxTextureDef, RafxExtents3D, RafxFormat};
use glyph_brush::ab_glyph::Rect;
use std::sync::Arc;
use crate::legion_support::LegionResources;
use rafx::assets::{AssetManager, AssetManagerRenderResource};

pub struct TextExtractJob {}

impl TextExtractJob {
    pub fn new() -> Self {
        Self {}
    }
}

impl ExtractJob
    for TextExtractJob
{
    fn extract(
        self: Box<Self>,
        extract_context: &RenderJobExtractContext,
        _frame_packet: &FramePacket,
        _views: &[&RenderView],
    ) -> Box<dyn PrepareJob> {
        profiling::scope!("Text Extract");
        let legion_resources = extract_context.render_resources.fetch::<LegionResources>();
        let asset_manager = extract_context.render_resources.fetch::<AssetManagerRenderResource>();

        let mut text_resource = legion_resources
            .get_mut::<TextResource>()
            .unwrap();
        let glyph_brush = text_resource.glyph_brush_mut();

        let mut image_update = None;
        let mut new_texture_size = None;

        let device_context = asset_manager.device_context();
        let max_image_dimension = 4096;
        let mut brush_action;

        //
        // Process the glyphs. We do it here because we may need to reallocate the texture, and we
        // need to cache the texture/vertex data for the next frame. We don't have write access to
        // it after the extract phase ends (although in the future we could have a way to move data
        // back to the game thread after a frame render finishes
        //
        loop {
            // Process the queued draw calls to produce vertex data (or nothing if we just need to
            // redraw what we had before
            brush_action = glyph_brush.process_queued(
                |rect, tex_data| unsafe {
                    let image_data = device_context.create_buffer(&RafxBufferDef::for_staging_buffer_data(tex_data, RafxResourceType::BUFFER)).unwrap();
                    image_data.copy_to_host_visible_buffer(tex_data).unwrap();
                    image_update = Some(TextImageUpdate {
                        upload_buffer: image_data,
                        upload_rectangle: rect
                    });
                },
                to_vertex,
            );

            // If the texture is too small, set new_texture_size to Some to resize it
            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall { suggested, .. }) => {
                    let (new_width, new_height) = if (suggested.0 > max_image_dimension
                        || suggested.1 > max_image_dimension)
                        && (glyph_brush.texture_dimensions().0 < max_image_dimension
                        || glyph_brush.texture_dimensions().1 < max_image_dimension)
                    {
                        (max_image_dimension, max_image_dimension)
                    } else {
                        suggested
                    };
                    log::info!("Resizing text glyph texture -> {}x{}", new_width, new_height);

                    // Recreate texture as a larger size to fit more
                    new_texture_size = Some(RafxExtents3D {
                        width: new_width,
                        height: new_height,
                        depth: 1
                    });

                    glyph_brush.resize_texture(new_width, new_height);
                }
            }
        }

        // If the texture didn't exist at all, set new_texture_size to force it to create the first
        // time we run through this code
        if text_resource.glyph_texture().is_none() {
            let (width, height) = text_resource.glyph_brush().texture_dimensions();
            new_texture_size = Some(RafxExtents3D {
                width,
                height,
                depth: 1
            });
        }

        // Recreate the texture if needed
        if let Some(extents) = new_texture_size {
            let texture = device_context.create_texture(&RafxTextureDef {
                extents,
                format: RafxFormat::R8_UNORM,
                ..Default::default()
            }).unwrap();

            let image = asset_manager.resources().insert_image(texture);
            let image_view = asset_manager.resources().get_or_create_image_view(&image, None).unwrap();

            *text_resource.glyph_texture_mut() = Some(image_view);
        }

        // Either cache the new vertex data, or reuse what we had before
        let vertex_data = match brush_action.unwrap() {
            BrushAction::Draw(vertices) => {
                let data = Arc::new(vertices);
                *text_resource.previous_texture_data_mut() = Some(data.clone());
                data
            },
            BrushAction::ReDraw => text_resource.previous_texture_data().as_ref().unwrap().clone()
        };

        let text_material = &extract_context
            .render_resources
            .fetch::<GameRendererStaticResources>()
            .text_material;
        let text_material_pass = asset_manager
            .get_material_pass_by_index(&text_material, 0)
            .unwrap();

        Box::new(TextPrepareJobImpl::new(
            text_material_pass,
            ExtractedTextData {
                image_update,
                vertex_data,
                texture: text_resource.glyph_texture().clone()
            },
        ))
    }

    fn feature_debug_name(&self) -> &'static str {
        TextRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        TextRenderFeature::feature_index()
    }
}


#[inline]
pub fn to_vertex(
    glyph_vertex: GlyphVertex
) -> TextVertex {
    let mut tex_coords = glyph_vertex.tex_coords;
    let pixel_coords = glyph_vertex.pixel_coords;
    let bounds = glyph_vertex.bounds;
    let extra = glyph_vertex.extra;

    let gl_bounds = bounds;

    let mut gl_rect = Rect {
        min: glyph_brush::ab_glyph::point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
        max: glyph_brush::ab_glyph::point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
    };

    // handle overlapping bounds, modify uv_rect to preserve texture aspect
    if gl_rect.max.x > gl_bounds.max.x {
        let old_width = gl_rect.width();
        gl_rect.max.x = gl_bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.min.x < gl_bounds.min.x {
        let old_width = gl_rect.width();
        gl_rect.min.x = gl_bounds.min.x;
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.max.y > gl_bounds.max.y {
        let old_height = gl_rect.height();
        gl_rect.max.y = gl_bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
    }
    if gl_rect.min.y < gl_bounds.min.y {
        let old_height = gl_rect.height();
        gl_rect.min.y = gl_bounds.min.y;
        tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
    }

    TextVertex {
        left_top: [gl_rect.min.x, gl_rect.max.y, extra.z],
        right_bottom: [gl_rect.max.x, gl_rect.min.y],
        tex_left_top: [tex_coords.min.x, tex_coords.max.y],
        tex_right_bottom: [tex_coords.max.x, tex_coords.min.y],
        color: extra.color,
    }
}
