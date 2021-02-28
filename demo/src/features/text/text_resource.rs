use crate::features::text::TextVertex;
use rafx::framework::{ResourceArc, ImageViewResource};
use std::sync::Arc;
//use glyph_brush::ab_glyph::{FontArc, Font};
use crate::assets::font::FontAsset;
use rafx::api::RafxResult;
use std::sync::atomic::{AtomicU32, Ordering};
use fnv::FnvHashMap;
use std::hash::{Hash, Hasher};
use rafx::distill::loader::handle::Handle;
use rafx::distill::loader::LoadHandle;
use rafx::distill::loader::handle::AssetHandle;

static NEXT_TYPEFACE_ID: AtomicU32 = AtomicU32::new(1);
static NEXT_FONT_ID: AtomicU32 = AtomicU32::new(1);

pub struct TypefaceId(pub u32);
pub struct FontId(pub u32);

// pub struct Typeface {
//     typeface_hash: u32,
// }

pub struct Font {
    typeface_hash: u32,
    font_handle: Handle<FontAsset>,
    size_in_pixels: f32,
    texture: Option<ResourceArc<ImageViewResource>>,
}

// impl Hash for Font {
//     fn hash<H: Hasher>(
//         &self,
//         state: &mut H,
//     ) {
//         self.font_bytes_hash.has
//     }
// }

pub struct AppendText<'a>(&'a mut TextResource);

impl<'a> AppendText<'a> {
    pub fn append(
        self,
        text: String,
        position: glam::Vec3,
        font: &Handle<FontAsset>,
        size: f32,
        color: glam::Vec4
    ) -> AppendText<'a> {
        self.0.append_previous_text(text, position, font, size, color)
    }
}


pub struct TextResource {
    //typefaces: FnvHashMap<TypefaceId, Typeface>,
    fonts: FnvHashMap<LoadHandle, Font>,
}

impl TextResource {
    pub fn new() -> Self {
        TextResource {
            //typefaces: Default::default(),
            fonts: Default::default(),
        }
    }

    pub fn add_text(
        &mut self,
        text: String,
        position: glam::Vec3,
        font: &Handle<FontAsset>,
        size: f32,
        color: glam::Vec4
    ) -> AppendText {
        let load_handle = font.load_handle();
        println!("text {}", text);

        AppendText(self)
    }

    pub fn append_previous_text(
        &mut self,
        text: String,
        position: glam::Vec3,
        font: &Handle<FontAsset>,
        size: f32,
        color: glam::Vec4
    ) -> AppendText {
        println!("text {}", text);

        AppendText(self)
    }

    // pub fn add_font(&mut self, font: &FontAsset, size_in_pixels: f32) -> RafxResult<()> {
    //     let font_id = NEXT_FONT_ID.fetch_add(1, Ordering::Relaxed);
    //
    //     fontdue::Font::from_bytes(&font.inner.data, fontdue::FontSettings {
    //         scale: size_in_pixels,
    //         ..Default::default()
    //     });
    //
    //     //let layout = fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYUp);
    //     layout.append(&[], fontdue::layout::TextStyle::new("hi", 20.0, 0));
    //
    //     self.font_id = Some(self.glyph_brush.add_font(f));
    //     Ok(())
    // }

    // pub fn glyph_brush(&self) -> &GlyphBrush<TextVertex> {
    //     &self.glyph_brush
    // }
    //
    // pub fn glyph_brush_mut(&mut self) -> &mut GlyphBrush<TextVertex> {
    //     &mut self.glyph_brush
    // }
    //
    // pub fn glyph_texture(&self) -> &Option<ResourceArc<ImageViewResource>> {
    //     &self.glyph_texture
    // }
    //
    // pub fn glyph_texture_mut(&mut self) -> &mut Option<ResourceArc<ImageViewResource>> {
    //     &mut self.glyph_texture
    // }
    //
    // pub fn previous_texture_data(&self) -> &Option<Arc<Vec<TextVertex>>> {
    //     &self.previous_texture_data
    // }
    //
    // pub fn previous_texture_data_mut(&mut self) -> &mut Option<Arc<Vec<TextVertex>>> {
    //     &mut self.previous_texture_data
    // }
}
