use metal::{MTLDataType, MTLResourceUsage, MTLArgumentAccess, MTLSamplerAddressMode, MTLRenderPipelineColorAttachmentDescriptor, MTLRenderPipelineColorAttachmentDescriptorArray, RenderPipelineColorAttachmentDescriptorArrayRef};
use crate::{RafxResourceType, RafxAddressMode, RafxDeviceInfo, RafxBlendState, RafxBlendStateTargets, MAX_RENDER_TARGET_ATTACHMENTS};

pub(crate) fn resource_type_mtl_data_type(
    resource_type: RafxResourceType,
) -> Option<MTLDataType> {
    if resource_type.intersects(RafxResourceType::UNIFORM_BUFFER | RafxResourceType::BUFFER | RafxResourceType::BUFFER_READ_WRITE) {
        Some(MTLDataType::Pointer)
    } else if resource_type.intersects(RafxResourceType::TEXTURE | RafxResourceType::TEXTURE_READ_WRITE) {
        Some(MTLDataType::Texture)
    } else if resource_type.intersects(RafxResourceType::SAMPLER) {
        Some(MTLDataType::Sampler)
    } else {
        None
    }
}

pub(crate) fn resource_type_mlt_resource_usage(
    resource_type: RafxResourceType
) -> MTLResourceUsage {
    let mut usage = MTLResourceUsage::empty();

    if resource_type.intersects(RafxResourceType::TEXTURE) {
        usage |= MTLResourceUsage::Sample;
    }

    if resource_type.intersects(RafxResourceType::TEXTURE_READ_WRITE) {
        usage |= MTLResourceUsage::Read | MTLResourceUsage::Write;
    }

    if resource_type.intersects(RafxResourceType::UNIFORM_BUFFER) {
        usage |= MTLResourceUsage::Read;
    }

    if resource_type.intersects(RafxResourceType::BUFFER) {
        usage |= MTLResourceUsage::Read;
    }

    if resource_type.intersects(RafxResourceType::BUFFER_READ_WRITE) {
        usage |= MTLResourceUsage::Read | MTLResourceUsage::Write;
    }

    if resource_type.intersects(RafxResourceType::TEXEL_BUFFER | RafxResourceType::TEXEL_BUFFER_READ_WRITE) {
        usage |= MTLResourceUsage::Sample;
    }

    usage
}

pub(crate) fn resource_type_mtl_argument_access(
    resource_type: RafxResourceType,
) -> MTLArgumentAccess {
    let usage = resource_type_mlt_resource_usage(resource_type);
    if usage.intersects(MTLResourceUsage::Write) {
        MTLArgumentAccess::ReadWrite
    } else {
        MTLArgumentAccess::ReadOnly
    }
}

pub(crate) fn address_mode_mtl_sampler_address_mode(
    address_mode: RafxAddressMode,
    device_info: &RafxDeviceInfo,
) -> MTLSamplerAddressMode {
    match address_mode {
        RafxAddressMode::Mirror => MTLSamplerAddressMode::MirrorRepeat,
        RafxAddressMode::Repeat => MTLSamplerAddressMode::Repeat,
        RafxAddressMode::ClampToEdge => MTLSamplerAddressMode::ClampToEdge,
        RafxAddressMode::ClampToBorder => if device_info.supports_clamp_to_border_color {
            MTLSamplerAddressMode::ClampToBorderColor
        } else {
            MTLSamplerAddressMode::ClampToZero
        }
    }
}

pub(crate) fn blend_def_to_attachment(
    blend_state: &RafxBlendState,
    attachments: &RenderPipelineColorAttachmentDescriptorArrayRef,
    color_attachment_count: usize,
) {
    blend_state.verify(color_attachment_count);
    // for (index, render_target) in blend_state.render_target_blend_states.iter().enumerate() {
    //     if (blend_state.render_target_mask.inter & (1<<index)) != 0 {
    //
    //     }
    // }

    if !blend_state.render_target_blend_states.is_empty() {
        for attachment_index in 0..MAX_RENDER_TARGET_ATTACHMENTS {
            if blend_state
                .render_target_mask
                .intersects(RafxBlendStateTargets::from_bits(1 << attachment_index).unwrap())
            {
                // Blend state can either be specified per target or once for all
                let def_index = if blend_state.independent_blend {
                    attachment_index
                } else {
                    0
                };

                let descriptor = attachments.object_at(attachment_index as _).unwrap();
                let def = &blend_state.render_target_blend_states[def_index];
                descriptor.set_blending_enabled(def.blend_enabled());
                descriptor.set_rgb_blend_operation(def.blend_op.into());
                descriptor.set_alpha_blend_operation(def.blend_op_alpha.into());
                descriptor.set_source_rgb_blend_factor(def.src_factor.into());
                descriptor.set_source_alpha_blend_factor(def.src_factor_alpha.into());
                descriptor.set_destination_rgb_blend_factor(def.dst_factor.into());
                descriptor.set_destination_alpha_blend_factor(def.dst_factor_alpha.into());
            };
        }
    }
}