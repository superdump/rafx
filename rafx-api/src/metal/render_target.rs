use crate::metal::{RafxTextureMetal, RafxDeviceContextMetal, RafxRawImageMetal};
use crate::{RafxTexture, RafxRenderTargetDef, RafxResult};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug)]
pub struct RafxRenderTargetMetalInner {
    // It's a RafxTextureMetal, but stored as RafxTexture so that we can return refs to it
    pub texture: RafxTexture,
    //is_undefined_layout: AtomicBool,
    pub render_target_def: RafxRenderTargetDef,
}

#[derive(Clone, Debug)]
pub struct RafxRenderTargetMetal {
    inner: Arc<RafxRenderTargetMetalInner>,
}

impl RafxRenderTargetMetal {
    pub fn render_target_def(&self) -> &RafxRenderTargetDef {
        &self.inner.render_target_def
    }

    pub fn texture(&self) -> &RafxTexture {
        &self.inner.texture
    }

    pub fn new(
        device_context: &RafxDeviceContextMetal,
        render_target_def: &RafxRenderTargetDef,
    ) -> RafxResult<Self> {
        unimplemented!();
        Self::from_existing(device_context, None, render_target_def)
    }

    pub fn from_existing(
        device_context: &RafxDeviceContextMetal,
        existing_image: Option<RafxRawImageMetal>,
        render_target_def: &RafxRenderTargetDef,
    ) -> RafxResult<Self> {
        render_target_def.verify();

        let mut texture_def = render_target_def.to_texture_def();

        let texture =
            RafxTextureMetal::from_existing(device_context, existing_image, &texture_def)?;

        let inner = RafxRenderTargetMetalInner {
            texture: RafxTexture::Metal(texture),
            //is_undefined_layout: AtomicBool::new(true),
            render_target_def: render_target_def.clone(),
        };

        Ok(RafxRenderTargetMetal {
            inner: Arc::new(inner),
        })
    }
}