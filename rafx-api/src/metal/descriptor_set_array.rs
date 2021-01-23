use crate::{RafxRootSignature, RafxDescriptorUpdate, RafxResult, RafxDescriptorSetArrayDef, RafxPipelineType, RafxShaderStageFlags, RafxBufferDef, RafxResourceType, RafxMemoryUsage, RafxQueueType, RafxDescriptorKey, RafxTextureBindType};
use crate::metal::{RafxDeviceContextMetal, DescriptorSetLayoutInfo, RafxBufferMetal};
use foreign_types_shared::ForeignTypeRef;

#[derive(Copy, Clone, Debug)]
pub struct RafxDescriptorSetHandleMetal {
    buffer: *mut metal_rs::MTLBuffer,
    offset: u32,
}

// for metal_rs::MTLBuffer
unsafe impl Send for RafxDescriptorSetHandleMetal {}
unsafe impl Sync for RafxDescriptorSetHandleMetal {}

impl RafxDescriptorSetHandleMetal {
    pub fn metal_buffer(&self) -> &metal_rs::BufferRef {
        use foreign_types_shared::ForeignTypeRef;
        unsafe {
            metal_rs::BufferRef::from_ptr(self.buffer)
        }
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }
}

pub struct ArgumentBufferData {
    buffer: RafxBufferMetal,
    encoder: metal_rs::ArgumentEncoder,
    stride: u32,
}

// for metal_rs::ArgumentEncoder
unsafe impl Send for ArgumentBufferData {}
unsafe impl Sync for ArgumentBufferData {}

pub struct RafxDescriptorSetArrayMetal {
    root_signature: RafxRootSignature,
    set_index: u32,
    argument_buffer_data: Option<ArgumentBufferData>,
}

impl RafxDescriptorSetArrayMetal {
    pub fn root_signature(&self) -> &RafxRootSignature {
        &self.root_signature
    }

    pub fn set_index(&self) -> u32 {
        self.set_index
    }

    pub fn metal_argument_buffer_and_offset(&self, index: u32) -> Option<(&metal_rs::BufferRef, u32)> {
        if let Some(argument_buffer_data) = &self.argument_buffer_data {
            Some((argument_buffer_data.buffer.metal_buffer(), index * argument_buffer_data.stride))
        } else {
            None
        }
    }

    pub fn handle(
        &self,
        index: u32,
    ) -> Option<RafxDescriptorSetHandleMetal> {
        self.metal_argument_buffer_and_offset(index).map(|(buffer, offset)| {
            RafxDescriptorSetHandleMetal {
                buffer: buffer.as_ptr(),
                offset,
            }
        })
    }

    pub(crate) fn new(
        device_context: &RafxDeviceContextMetal,
        descriptor_set_array_def: &RafxDescriptorSetArrayDef,
    ) -> RafxResult<Self> {
        let root_signature = descriptor_set_array_def
            .root_signature
            .metal_root_signature()
            .unwrap()
            .clone();

        let layout_index = descriptor_set_array_def.set_index as usize;
        let layout = &root_signature.inner.layouts[layout_index];

        let argument_descriptors = &root_signature.inner.argument_descriptors[layout_index];
        let immutable_samplers = &layout.immutable_samplers;
        let argument_buffer_data = if !argument_descriptors.is_empty() || !immutable_samplers.is_empty() {
            let array = metal_rs::Array::from_owned_slice(&argument_descriptors);
            let encoder = device_context.device().new_argument_encoder(array);

            let required_alignment = 256;
            assert!(required_alignment >= encoder.alignment() as _);
            let stride = rafx_base::memory::round_size_up_to_alignment_u32(encoder.encoded_length() as _, required_alignment);
            let total_buffer_size = stride * descriptor_set_array_def.array_length as u32;

            let buffer = device_context.create_buffer(&RafxBufferDef {
                size: total_buffer_size as u64,
                alignment: required_alignment,
                resource_type: RafxResourceType::ARGUMENT_BUFFER,
                memory_usage: RafxMemoryUsage::CpuToGpu,
                queue_type: RafxQueueType::Graphics,
                always_mapped: true,
                ..Default::default()
            })?;

            // Bind static samplers
            for immutable_sampler in immutable_samplers {
                for array_index in 0..descriptor_set_array_def.array_length {
                    encoder.set_argument_buffer(buffer.metal_buffer(), (array_index as u32 * stride) as _);

                    let samplers : Vec<_> = immutable_sampler.samplers.iter().map(|x| x.metal_sampler()).collect();
                    encoder.set_sampler_states(immutable_sampler.argument_buffer_id, &samplers);
                }
            }

            Some(ArgumentBufferData {
                encoder,
                buffer,
                stride
            })
        } else {
            None
        };

        Ok(RafxDescriptorSetArrayMetal {
            root_signature: RafxRootSignature::Metal(root_signature),
            set_index: descriptor_set_array_def.set_index,
            argument_buffer_data,
        })
    }

    pub fn update_descriptor_set(
        &mut self,
        descriptor_updates: &[RafxDescriptorUpdate],
    ) -> RafxResult<()> {
        for update in descriptor_updates {
            self.queue_descriptor_set_update(update)?;
        }
        self.flush_descriptor_set_updates()
    }

    pub fn flush_descriptor_set_updates(&mut self) -> RafxResult<()> {
        // Don't need to do anything on flush
        Ok(())
    }

    pub fn queue_descriptor_set_update(
        &mut self,
        update: &RafxDescriptorUpdate,
    ) -> RafxResult<()> {
        let root_signature = self.root_signature.metal_root_signature().unwrap();
        let layout: &DescriptorSetLayoutInfo =
            &root_signature.inner.layouts[self.set_index as usize];
        let descriptor_index = match &update.descriptor_key {
            RafxDescriptorKey::Name(name) => {
                let descriptor_index = root_signature.find_descriptor_by_name(name);
                if let Some(descriptor_index) = descriptor_index {
                    let set_index = root_signature
                        .descriptor(descriptor_index)
                        .unwrap()
                        .set_index;
                    if set_index == self.set_index {
                        descriptor_index
                    } else {
                        return Err(format!(
                            "Found descriptor {:?} but it's set_index ({:?}) does not match the set ({:?})",
                            &update.descriptor_key,
                            set_index,
                            self.set_index
                        ))?;
                    }
                } else {
                    return Err(format!(
                        "Could not find descriptor {:?}",
                        &update.descriptor_key
                    ))?;
                }
            }
            RafxDescriptorKey::Binding(binding) => layout
                .binding_to_descriptor_index
                .get(binding)
                .copied()
                .ok_or_else(|| format!("Could not find descriptor {:?}", update.descriptor_key,))?,
            RafxDescriptorKey::DescriptorIndex(descriptor_index) => *descriptor_index,
            RafxDescriptorKey::Undefined => {
                return Err("Passed RafxDescriptorKey::Undefined to update_descriptor_set()")?
            }
        };

        let descriptor = root_signature.descriptor(descriptor_index).unwrap();

        let argument_buffer = self.argument_buffer_data.as_ref().unwrap();
        argument_buffer.encoder.set_argument_buffer(
            argument_buffer.buffer.metal_buffer(),
            (update.array_index * argument_buffer.stride) as _
        );

        log::trace!(
            "update descriptor set {:?} (set_index: {:?} binding: {} name: {:?} type: {:?} array_index: {} arg buffer id: {})",
            update.descriptor_key,
            descriptor.set_index,
            descriptor.binding,
            descriptor.name,
            descriptor.resource_type,
            update.array_index,
            descriptor.argument_buffer_id,
        );

        match descriptor.resource_type {
            RafxResourceType::SAMPLER => {
                let samplers = update.elements.samplers.ok_or_else(||
                    format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the samplers element list was None",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    )
                )?;

                let begin_index =
                    descriptor.argument_buffer_id as usize + update.dst_element_offset as usize;
                assert!(update.dst_element_offset + samplers.len() as u32 <= descriptor.element_count);

                let mut next_index = begin_index;
                for sampler in samplers {
                    argument_buffer.encoder.set_sampler_state(next_index as _, sampler.metal_sampler().unwrap().metal_sampler());
                    next_index += 1;
                }
            }
            RafxResourceType::TEXTURE | RafxResourceType::TEXTURE_READ_WRITE => {
                let textures = update.elements.textures.ok_or_else(||
                    format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the texture element list was None",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    )
                )?;

                // Defaults to UavMipSlice(0) for TEXTURE_READ_WRITE and Srv for TEXTURE
                let texture_bind_type = if descriptor.resource_type == RafxResourceType::TEXTURE_READ_WRITE {
                    update
                        .texture_bind_type
                        .unwrap_or(RafxTextureBindType::UavMipSlice(0))
                } else {
                    update
                        .texture_bind_type
                        .unwrap_or(RafxTextureBindType::Srv)
                };

                let begin_index =
                    descriptor.argument_buffer_id as usize + update.dst_element_offset as usize;
                assert!(update.dst_element_offset + textures.len() as u32 <= descriptor.element_count);

                let mut next_index = begin_index;
                if let RafxTextureBindType::UavMipSlice(slice) = texture_bind_type {
                    for texture in textures {
                        let uav_views = texture.metal_texture().unwrap().metal_mip_level_uav_views();
                        let uav_view = uav_views.get(slice as usize).ok_or_else(|| format!(
                            "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the chosen mip slice {} exceeds the mip count of {} in the image",
                            update.descriptor_key,
                            descriptor.set_index,
                            descriptor.binding,
                            descriptor.name,
                            descriptor.resource_type,
                            slice,
                            uav_views.len()
                        ))?;

                        argument_buffer.encoder.set_texture(next_index as _, uav_view);
                        next_index += 1;
                    }
                } else if texture_bind_type == RafxTextureBindType::UavMipChain {
                    let texture = textures.first().unwrap();

                    let uav_views = texture.metal_texture().unwrap().metal_mip_level_uav_views();
                    if uav_views.len() > descriptor.element_count as usize {
                        Err(format!(
                            "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) using UavMipChain but the mip chain has {} images and the descriptor has {} elements",
                            update.descriptor_key,
                            descriptor.set_index,
                            descriptor.binding,
                            descriptor.name,
                            descriptor.resource_type,
                            uav_views.len(),
                            descriptor.element_count
                        ))?;
                    }

                    for uav_view in uav_views {
                        argument_buffer.encoder.set_texture(next_index as _, uav_view);
                        next_index += 1;
                    }
                } else if texture_bind_type == RafxTextureBindType::Srv || texture_bind_type == RafxTextureBindType::SrvStencil {
                    for texture in textures {
                        argument_buffer.encoder.set_texture(next_index as _, texture.metal_texture().unwrap().metal_texture());
                        next_index += 1;
                    }
                } else {
                    Err(format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but texture_bind_type {:?} was unexpected for this kind of resource",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                        update.texture_bind_type
                    ))?;
                }
            }
            RafxResourceType::UNIFORM_BUFFER
            | RafxResourceType::BUFFER
            | RafxResourceType::BUFFER_READ_WRITE => {
                let buffers = update.elements.buffers.ok_or_else(||
                    format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the buffers element list was None",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type,
                    )
                )?;

                let begin_index =
                    descriptor.argument_buffer_id as usize + update.dst_element_offset as usize;
                assert!(update.dst_element_offset + buffers.len() as u32 <= descriptor.element_count);

                // Modify the update data
                let mut next_index = begin_index;
                for (buffer_index, buffer) in buffers.iter().enumerate() {
                    let offset = update.elements.buffer_offset_sizes.map(|x| x[buffer_index].offset).unwrap_or(0);
                    //println!("arg buffer index: {} offset {} buffer {:?}", next_index, offset, buffer.metal_buffer().unwrap().metal_buffer());

                    argument_buffer.encoder.set_buffer(
                        next_index as _,
                        buffer.metal_buffer().unwrap().metal_buffer(),
                        offset
                    );
                    next_index += 1;
                }
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
}

impl std::fmt::Debug for RafxDescriptorSetArrayMetal {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("RafxDescriptorSetArrayMetal")
            //.field("first_descriptor_set", &self.descriptor_sets[0])
            //.field("root_signature", &self.root_signature)
            //.field("set_index", &self.set_index)
            //.field("pending_write_count", &self.pending_writes.len())
            .finish()
    }
}