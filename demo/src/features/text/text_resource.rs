use glyph_brush::{GlyphBrush, GlyphBrushBuilder, FontId};
use crate::features::text::TextVertex;
use rafx::framework::{ResourceArc, ImageViewResource};
use std::sync::Arc;
use glyph_brush::ab_glyph::{FontArc, Font};
use crate::assets::font::FontAsset;
use rafx::api::RafxResult;

pub struct TextResource {
    glyph_brush: GlyphBrush<TextVertex>,
    glyph_texture: Option<ResourceArc<ImageViewResource>>,
    previous_texture_data: Option<Arc<Vec<TextVertex>>>,
    font_id: Option<FontId>,
}

impl TextResource {
    pub fn new() -> Self {
        let fonts = Vec::<FontArc>::default();
        let glyph_brush = GlyphBrushBuilder::using_fonts(fonts)
            .multithread(false)
            .build();

        TextResource {
            glyph_brush,
            glyph_texture: None,
            previous_texture_data: None,
            font_id: None,
        }
    }

    pub fn add_font(&mut self, font: &FontAsset) -> RafxResult<()> {
        let f = FontArc::try_from_vec(font.inner.data.clone()).map_err(|x| x.to_string())?;
        let a = f.glyph_id('x');
        let b = f.glyph_id('b');
        let c = f.glyph_id('h');
        let d = f.glyph_id('6');
        self.font_id = Some(self.glyph_brush.add_font(f));
        Ok(())
    }

    pub fn glyph_brush(&self) -> &GlyphBrush<TextVertex> {
        &self.glyph_brush
    }

    pub fn glyph_brush_mut(&mut self) -> &mut GlyphBrush<TextVertex> {
        &mut self.glyph_brush
    }

    pub fn glyph_texture(&self) -> &Option<ResourceArc<ImageViewResource>> {
        &self.glyph_texture
    }

    pub fn glyph_texture_mut(&mut self) -> &mut Option<ResourceArc<ImageViewResource>> {
        &mut self.glyph_texture
    }

    pub fn previous_texture_data(&self) -> &Option<Arc<Vec<TextVertex>>> {
        &self.previous_texture_data
    }

    pub fn previous_texture_data_mut(&mut self) -> &mut Option<Arc<Vec<TextVertex>>> {
        &mut self.previous_texture_data
    }
}
