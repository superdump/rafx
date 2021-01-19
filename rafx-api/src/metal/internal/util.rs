use metal::{MTLDataType, MTLResourceUsage, MTLArgumentAccess, MTLSamplerAddressMode};
use crate::{RafxResourceType, RafxAddressMode, RafxDeviceInfo};

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