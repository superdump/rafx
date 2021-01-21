use crate::{RafxTextureDef, RafxResult, RafxResourceType};
use crate::metal::RafxDeviceContextMetal;

#[derive(Debug)]
pub enum RafxRawImageMetal {
    Owned(metal_rs::Texture),
    Ref(metal_rs::Texture),
    //Null,
}

impl RafxRawImageMetal {
    pub fn metal_texture(&self) -> &metal_rs::TextureRef {
        match self {
            RafxRawImageMetal::Owned(owned) => owned.as_ref(),
            RafxRawImageMetal::Ref(r) => r.as_ref(),
            //RafxRawImageMetal::Null => None
        }
    }
}

/// Holds the vk::Image and allocation as well as a few vk::ImageViews depending on the
/// provided RafxResourceType in the texture_def.
#[derive(Debug)]
pub struct RafxTextureMetal {
    device_context: RafxDeviceContextMetal,
    texture_def: RafxTextureDef,
    image: RafxRawImageMetal,
}

impl RafxTextureMetal {
    pub fn texture_def(&self) -> &RafxTextureDef {
        &self.texture_def
    }

    pub fn metal_texture(&self) -> &metal_rs::TextureRef {
        self.image.metal_texture()
    }

    pub fn new(
        device_context: &RafxDeviceContextMetal,
        texture_def: &RafxTextureDef,
    ) -> RafxResult<RafxTextureMetal> {
        Self::from_existing(device_context, None, texture_def)
    }

    // This path is mostly so we can wrap a provided swapchain image
    pub fn from_existing(
        device_context: &RafxDeviceContextMetal,
        existing_image: Option<RafxRawImageMetal>,
        texture_def: &RafxTextureDef,
    ) -> RafxResult<RafxTextureMetal> {
        texture_def.verify();

        let image = if let Some(existing_image) = existing_image {
            existing_image
        } else {
            unimplemented!();
        };

        // if texture_def.resource_type.intersects(RafxResourceType::TEXTURE_READ_WRITE) {
        //     for _ in 0..texture_def.mip_count {
        //         raw_image.metal_texture().unwrap().new_texture_view_from_slice(
        //             format,
        //             texture_type,
        //             mips,
        //             range
        //         )
        //     }
        // }

        Ok(RafxTextureMetal {
            texture_def: texture_def.clone(),
            device_context: device_context.clone(),
            image
        })
    }
}