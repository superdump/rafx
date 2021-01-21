use crate::{RafxRootSignature, RafxDescriptorUpdate, RafxResult, RafxDescriptorSetArrayDef, RafxPipelineType, RafxShaderStageFlags, RafxBufferDef, RafxResourceType, RafxMemoryUsage, RafxQueueType, RafxDescriptorKey, RafxTextureBindType};
use crate::metal::{RafxDeviceContextMetal, DescriptorSetLayoutInfo, RafxBufferMetal};

// struct DescriptorUpdateData {
//     // one per set * elements in each descriptor
//     buffers: Vec<Option<metal::Buffer>>,
//     textures: Vec<Option<metal::Texture>>,
//     samplers: Vec<Option<metal::SamplerState>>,
// }
//
// impl DescriptorUpdateData {
//     fn new(
//         descriptor_set_array_def: &RafxDescriptorSetArrayDef,
//         layout: &DescriptorSetLayoutInfo
//     ) -> Self {
//         let array_length = descriptor_set_array_def.array_length;
//         DescriptorUpdateData {
//             buffers: vec![None; array_length * layout.buffer_count as usize],
//             textures: vec![None; array_length * layout.texture_count as usize],
//             samplers: vec![None; array_length * layout.sampler_count as usize],
//         }
//     }
// }

#[derive(Copy, Clone, Debug)]
pub struct RafxDescriptorSetHandleMetal;

pub struct ArgumentBufferData {
    buffer: RafxBufferMetal,
    encoder: metal_rs::ArgumentEncoder,
    stride: u32,
}

pub struct RafxDescriptorSetArrayMetal {
    root_signature: RafxRootSignature,
    set_index: u32,
    //update_data: DescriptorUpdateData,
    argument_buffer_data: Option<ArgumentBufferData>,
}

impl RafxDescriptorSetArrayMetal {
    pub fn root_signature(&self) -> &RafxRootSignature {
        &self.root_signature
    }

    pub fn set_index(&self) -> u32 {
        self.set_index
    }

    pub fn handle(
        &self,
        index: u32,
    ) -> Option<RafxDescriptorSetHandleMetal> {
        unimplemented!();
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
        //let update_data = DescriptorUpdateData::new(descriptor_set_array_def, &layout);

        // for descriptor_index in layout.descriptors {
        //     let descriptor = root_signature.descriptor(descriptor_index).unwrap();
        //     if descriptor.immutable_sampler.is_some() {
        //         for i in des
        //     }
        // }

        let argument_descriptors = &root_signature.inner.argument_descriptors[layout_index];
        let immutable_samplers = &layout.immutable_samplers;
        let argument_buffer_data = if !argument_descriptors.is_empty() || !immutable_samplers.is_empty() {
            // let shader_stages = if root_signature.pipeline_type() == RafxPipelineType::Compute {
            //     RafxShaderStageFlags::COMPUTE
            // } else {
            //     RafxShaderStageFlags::VERTEX | RafxShaderStageFlags::FRAGMENT
            // };

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

                    //let samplers = immutable_sampler.samplers.iter().map(|x| x.metal_sampler()).collect();
                    //encoder.set_sampler_states(0, samplers);
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
            argument_buffer_data
            //descriptor_sets,
            //update_data,
            //pending_writes:
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
        /*
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

        //let descriptor_index = descriptor_index.ok_or_else(|| format!("Could not find descriptor {:?}", &update.descriptor_key))?;
        let descriptor = root_signature.descriptor(descriptor_index).unwrap();

        // let descriptor_first_update_data = descriptor.update_data_offset_in_set.unwrap()
        //     + (layout.update_data_count_per_set * update.array_index);

        //let mut descriptor_set_writes = Vec::default();

        // let vk_set = self.descriptor_sets[update.array_index as usize];
        // let write_descriptor_builder = vk::WriteDescriptorSet::builder()
        //     .dst_set(vk_set)
        //     .dst_binding(descriptor.binding)
        //     .dst_array_element(update.dst_element_offset)
        //     .descriptor_type(descriptor.vk_type);

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
                // let begin_index =
                //     (descriptor_first_update_data + update.dst_element_offset) as usize;
                // assert!(begin_index + samplers.len() <= self.update_data.update_data_count);
                //
                // // Modify the update data
                // let mut next_index = begin_index;
                // for sampler in samplers {
                //     let image_info = &mut self.update_data.image_infos[next_index];
                //     next_index += 1;
                //
                //     image_info.sampler = sampler.vk_sampler().unwrap().vk_sampler();
                // }
                //
                // // Queue a descriptor write
                // self.pending_writes.push(
                //     write_descriptor_builder
                //         .image_info(&self.update_data.image_infos[begin_index..next_index])
                //         .build(),
                // );

                let begin_index =
                    (descriptor.argument_buffer_id + update.dst_element_offset) as usize;
                assert!(update.dst_element_offset + samplers.len() <= descriptor.element_count as usize);

                // Modify the update data
                let mut next_index = begin_index;
                for sampler in samplers {
                    argument_buffer.encoder.set_sampler_state(next_index as _, sampler.metal_sampler().unwrap().metal_sampler());
                    next_index += 1;
                }
            }
            RafxResourceType::COMBINED_IMAGE_SAMPLER => {
                if !descriptor.has_immutable_sampler {
                    Err(format!(
                        "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the sampler is NOT immutable. This is not currently supported.",
                        update.descriptor_key,
                        descriptor.set_index,
                        descriptor.binding,
                        descriptor.name,
                        descriptor.resource_type
                    ))?;
                }

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

                let begin_index =
                    (descriptor.argument_buffer_id + update.dst_element_offset) as usize;
                assert!(begin_index + textures.len() <= self.update_data.update_data_count);

                let texture_bind_type =
                    update.texture_bind_type.unwrap_or(RafxTextureBindType::Srv);

                // Modify the update data
                let mut next_index = begin_index;
                for texture in textures {
                    let image_info = &mut self.update_data.image_infos[next_index];
                    next_index += 1;

                    if texture_bind_type == RafxTextureBindType::SrvStencil {
                        image_info.image_view = texture.vk_texture().unwrap().vk_srv_view_stencil().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) as RafxTextureBindType::SrvStencil but there is no srv_stencil view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    } else if texture_bind_type == RafxTextureBindType::Srv {
                        image_info.image_view = texture.vk_texture().unwrap().vk_srv_view().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) as RafxTextureBindType::Srv but there is no srv_stencil view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
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

                    image_info.image_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .image_info(&self.update_data.image_infos[begin_index..next_index])
                        .build(),
                );
            }
            RafxResourceType::TEXTURE => {
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
                let begin_index =
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + textures.len() <= self.update_data.update_data_count);

                let texture_bind_type =
                    update.texture_bind_type.unwrap_or(RafxTextureBindType::Srv);

                // Modify the update data
                let mut next_index = begin_index;
                for texture in textures {
                    let image_info = &mut self.update_data.image_infos[next_index];
                    next_index += 1;

                    if texture_bind_type == RafxTextureBindType::SrvStencil {
                        image_info.image_view = texture.vk_texture().unwrap().vk_srv_view_stencil().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) as RafxTextureBindType::SrvStencil but there is no srv_stencil view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    } else if texture_bind_type == RafxTextureBindType::Srv {
                        image_info.image_view = texture.vk_texture().unwrap().vk_srv_view().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) as RafxTextureBindType::Srv but there is no srv view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
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

                    image_info.image_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .image_info(&self.update_data.image_infos[begin_index..next_index])
                        .build(),
                );
            }
            RafxResourceType::TEXTURE_READ_WRITE => {
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
                let begin_index =
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + textures.len() <= self.update_data.update_data_count);

                // Modify the update data
                let mut next_index = begin_index;

                let texture_bind_type = update
                    .texture_bind_type
                    .unwrap_or(RafxTextureBindType::UavMipSlice(0));

                if let RafxTextureBindType::UavMipSlice(slice) = texture_bind_type {
                    for texture in textures {
                        let image_info = &mut self.update_data.image_infos[next_index];
                        next_index += 1;

                        let image_views = texture.vk_texture().unwrap().vk_uav_views();
                        let image_view = *image_views.get(slice as usize).ok_or_else(|| format!(
                            "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but the chosen mip slice {} exceeds the mip count of {} in the image",
                            update.descriptor_key,
                            descriptor.set_index,
                            descriptor.binding,
                            descriptor.name,
                            descriptor.resource_type,
                            slice,
                            image_views.len()
                        ))?;
                        image_info.image_view = image_view;

                        image_info.image_layout = vk::ImageLayout::GENERAL;
                    }
                } else if texture_bind_type == RafxTextureBindType::UavMipChain {
                    let texture = textures.first().unwrap();

                    let image_views = texture.vk_texture().unwrap().vk_uav_views();
                    if image_views.len() > descriptor.element_count as usize {
                        Err(format!(
                            "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) using UavMipChain but the mip chain has {} images and the descriptor has {} elements",
                            update.descriptor_key,
                            descriptor.set_index,
                            descriptor.binding,
                            descriptor.name,
                            descriptor.resource_type,
                            image_views.len(),
                            descriptor.element_count
                        ))?;
                    }

                    for image_view in image_views {
                        let image_info = &mut self.update_data.image_infos[next_index];
                        next_index += 1;

                        image_info.image_view = *image_view;
                        image_info.image_layout = vk::ImageLayout::GENERAL;
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

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .image_info(&self.update_data.image_infos[begin_index..next_index])
                        .build(),
                );
            }
            RafxResourceType::UNIFORM_BUFFER
            | RafxResourceType::BUFFER
            | RafxResourceType::BUFFER_READ_WRITE => {
                if descriptor.vk_type == vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC {
                    //TODO: Add support for dynamic uniforms
                    unimplemented!();
                }

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
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + buffers.len() <= self.update_data.update_data_count);

                // Modify the update data
                let mut next_index = begin_index;
                for (buffer_index, buffer) in buffers.iter().enumerate() {
                    let buffer_info = &mut self.update_data.buffer_infos[next_index];
                    next_index += 1;

                    buffer_info.buffer = buffer.vk_buffer().unwrap().vk_buffer();
                    buffer_info.offset = 0;
                    buffer_info.range = vk::WHOLE_SIZE;

                    if let Some(offset_size) = update.elements.buffer_offset_sizes {
                        if offset_size[buffer_index].offset != 0 {
                            buffer_info.offset = offset_size[buffer_index].offset;
                        }

                        if offset_size[buffer_index].size != 0 {
                            buffer_info.range = offset_size[buffer_index].size;
                        }
                    }
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .buffer_info(&self.update_data.buffer_infos[begin_index..next_index])
                        .build(),
                );
            }
            RafxResourceType::TEXEL_BUFFER | RafxResourceType::TEXEL_BUFFER_READ_WRITE => {
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
                    (descriptor_first_update_data + update.dst_element_offset) as usize;
                assert!(begin_index + buffers.len() <= self.update_data.update_data_count);

                // Modify the update data
                let mut next_index = begin_index;
                for buffer in buffers {
                    let buffer_view = &mut self.update_data.buffer_views[next_index];
                    next_index += 1;

                    if descriptor.resource_type == RafxResourceType::TEXEL_BUFFER {
                        *buffer_view = buffer.vk_buffer().unwrap().vk_uniform_texel_view().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but there was no uniform texel view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    } else {
                        *buffer_view = buffer.vk_buffer().unwrap().vk_storage_texel_view().ok_or_else(|| {
                            format!(
                                "Tried to update binding {:?} (set: {:?} binding: {} name: {:?} type: {:?}) but there was no storage texel view",
                                update.descriptor_key,
                                descriptor.set_index,
                                descriptor.binding,
                                descriptor.name,
                                descriptor.resource_type,
                            )
                        })?;
                    };
                }

                // Queue a descriptor write
                self.pending_writes.push(
                    write_descriptor_builder
                        .texel_buffer_view(&self.update_data.buffer_views[begin_index..next_index])
                        .build(),
                );
            }
            _ => unimplemented!(),
        }
        */
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