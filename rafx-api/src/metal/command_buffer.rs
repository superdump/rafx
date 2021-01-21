use crate::{RafxCommandBufferDef, RafxResult, RafxColorRenderTargetBinding, RafxDepthRenderTargetBinding, RafxVertexBufferBinding, RafxBufferBarrier, RafxTextureBarrier, RafxRenderTargetBarrier, RafxCmdCopyBufferToTextureParams, RafxCmdBlitParams, RafxIndexBufferBinding, RafxResourceState, RafxLoadOp, RafxStoreOp, RafxQueueType, RafxPipelineType};
use crate::metal::{RafxCommandPoolMetal, RafxPipelineMetal, RafxDescriptorSetArrayMetal, RafxRootSignatureMetal, RafxDescriptorSetHandleMetal, RafxBufferMetal, RafxTextureMetal, RafxQueueMetal, BarrierFlagsMetal, RafxRenderTargetMetal};
use std::sync::atomic::{AtomicPtr, AtomicU8, AtomicBool, AtomicU64};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use fnv::FnvHashSet;
use metal_rs::{MTLRenderStages, MTLPrimitiveType, MTLCommandBuffer, MTLRenderCommandEncoder, MTLComputeCommandEncoder, MTLBlitCommandEncoder, MTLResourceUsage};

// Mutable state stored in a lock. (Hopefully we can optimize away the lock later)
#[derive(Debug)]
pub struct RafxCommandBufferMetalInner {
    render_targets_to_make_readable: FnvHashSet<RafxRenderTargetMetal>,
}

#[derive(Debug)]
pub struct RafxCommandBufferMetal {
    queue: RafxQueueMetal,
    command_buffer: AtomicPtr<MTLCommandBuffer>,
    inner: Mutex<RafxCommandBufferMetalInner>,
    render_encoder: AtomicPtr<MTLRenderCommandEncoder>,
    compute_encoder: AtomicPtr<MTLComputeCommandEncoder>,
    blit_encoder: AtomicPtr<MTLBlitCommandEncoder>,
    last_pipeline_type: AtomicU8, // RafxPipelineType
    primitive_type: AtomicU8, // MTLPrimitiveType
}

impl Drop for RafxCommandBufferMetal {
    fn drop(&mut self) {
        // If these contain valid pointers, put the pointer into the wrapper and drop it
        let _ = self.swap_command_buffer(None);
        let _ = self.swap_render_encoder(None);
        let _ = self.swap_compute_encoder(None);
        let _ = self.swap_blit_encoder(None);
    }
}

impl RafxCommandBufferMetal {
    pub fn new(
        command_pool: &RafxCommandPoolMetal,
        _command_buffer_def: &RafxCommandBufferDef,
    ) -> RafxResult<RafxCommandBufferMetal> {
        let inner = RafxCommandBufferMetalInner {
            render_targets_to_make_readable: Default::default()
        };

        Ok(RafxCommandBufferMetal {
            queue: command_pool.queue().clone(),
            command_buffer: Default::default(),
            render_encoder: Default::default(),
            compute_encoder: Default::default(),
            blit_encoder: Default::default(),
            last_pipeline_type: AtomicU8::new(u8::max_value()),
            primitive_type: Default::default(),
            inner: Mutex::new(inner),
        })
    }


    pub fn begin(&self) -> RafxResult<()> {
        let command_buffer = self.queue.metal_queue().new_command_buffer();
        self.swap_command_buffer(Some(command_buffer.to_owned()));
        self.last_pipeline_type.store(0, Ordering::Relaxed);
        Ok(())
    }

    pub fn end(&self) -> RafxResult<()> {
        self.end_current_encoders(true)
    }

    pub fn return_to_pool(&self) -> RafxResult<()> {
        // Returning to pool means the command buffer no longer needs to stay valid, so drop the
        // current one
        self.swap_command_buffer(None);
        Ok(())
    }

    pub fn cmd_bind_render_targets(
        &self,
        color_targets: &[RafxColorRenderTargetBinding],
        depth_target: Option<RafxDepthRenderTargetBinding>,
    ) -> RafxResult<()> {
        // if self.has_active_renderpass.load(Ordering::Relaxed) {
        //     self.cmd_unbind_render_targets()?;
        // }

        if color_targets.is_empty() && depth_target.is_none() {
            Err("No color or depth target supplied to cmd_bind_render_targets")?;
        }

        let descriptor = metal_rs::RenderPassDescriptor::new();
        for (i, color_target) in color_targets.iter().enumerate() {
            let color_descriptor = descriptor.color_attachments().object_at(i as _).unwrap();
            let texture = color_target.render_target.texture().metal_texture().unwrap();
            color_descriptor.set_texture(Some(texture.metal_texture()));
            color_descriptor.set_level(color_target.mip_slice.unwrap_or(0) as _);
            if color_target.array_slice.is_some() {
                if texture.texture_def().extents.depth > 1 {
                    color_descriptor.set_depth_plane(color_target.array_slice.unwrap() as _);
                } else {
                    color_descriptor.set_slice(color_target.array_slice.unwrap() as _);
                }
            }

            color_descriptor.set_load_action(color_target.load_op.into());
            let store_action = super::util::color_render_target_binding_mtl_store_op(color_target);
            color_descriptor.set_store_action(store_action);

            if color_target.load_op == RafxLoadOp::Clear {
                color_descriptor.set_clear_color(color_target.clear_value.into());
            }
        }

        if let Some(depth_target) = depth_target {
            let depth_descriptor = descriptor.depth_attachment().unwrap();
            let texture = depth_target.render_target.texture().metal_texture().unwrap();

            depth_descriptor.set_texture(Some(texture.metal_texture()));
            depth_descriptor.set_level(depth_target.mip_slice.unwrap_or(0) as _);
            depth_descriptor.set_slice(depth_target.array_slice.unwrap_or(0) as _);
            depth_descriptor.set_load_action(depth_target.depth_load_op.into());
            depth_descriptor.set_store_action(depth_target.depth_store_op.into());

            let has_stencil = texture.texture_def().format.has_stencil();
            if has_stencil {
                let stencil_descriptor = descriptor.stencil_attachment().unwrap();
                stencil_descriptor.set_texture(Some(texture.metal_texture()));
                stencil_descriptor.set_level(depth_target.mip_slice.unwrap_or(0) as _);
                stencil_descriptor.set_slice(depth_target.array_slice.unwrap_or(0) as _);
                stencil_descriptor.set_load_action(depth_target.stencil_load_op.into());
                stencil_descriptor.set_store_action(depth_target.stencil_store_op.into());
            } else {
                //let stencil_descriptor = descriptor.stencil_attachment().unwrap();
                //stencil_descriptor.set_load_action(RafxStoreOp::DontCare.into());
                //stencil_descriptor.set_store_action(RafxStoreOp::DontCare.into());
            }
        } else {
            // let depth_descriptor = descriptor.depth_attachment().unwrap();
            // depth_descriptor.set_load_action(RafxStoreOp::DontCare.into());
            // depth_descriptor.set_store_action(RafxStoreOp::DontCare.into());
            // let stencil_descriptor = descriptor.stencil_attachment().unwrap();
            // stencil_descriptor.set_load_action(RafxStoreOp::DontCare.into());
            // stencil_descriptor.set_store_action(RafxStoreOp::DontCare.into());
        }

        // end encoders
        self.end_current_encoders(false)?;
        let render_encoder = self.metal_command_buffer().unwrap().new_render_command_encoder(descriptor);
        self.swap_render_encoder(Some(render_encoder.to_owned()));
        self.wait_for_barriers()?;
        // set heaps?

        Ok(())
    }

    pub(crate) fn end_current_encoders(&self, force_barrier: bool) -> RafxResult<()> {
        let barrier_flags = self.queue.barrier_flags();

        if let Some(render_encoder) = self.swap_render_encoder(None) {
            if !barrier_flags.is_empty() || force_barrier {
                render_encoder.update_fence(self.queue.metal_fence(), MTLRenderStages::Fragment);
                self.queue.add_barrier_flags(BarrierFlagsMetal::FENCE);
            }

            render_encoder.end_encoding();
        } else if let Some(compute_encoder) = self.swap_compute_encoder(None) {
            if !barrier_flags.is_empty() || force_barrier {
                compute_encoder.update_fence(self.queue.metal_fence());
                self.queue.add_barrier_flags(BarrierFlagsMetal::FENCE);
            }

            compute_encoder.end_encoding();
        } else if let Some(blit_encoder) = self.swap_blit_encoder(None) {
            if !barrier_flags.is_empty() || force_barrier {
                blit_encoder.update_fence(self.queue.metal_fence());
                self.queue.add_barrier_flags(BarrierFlagsMetal::FENCE);
            }

            blit_encoder.end_encoding();
        }

        Ok(())
    }

    fn wait_for_barriers(&self) -> RafxResult<()> {
        let barrier_flags = self.queue.barrier_flags();
        if barrier_flags.is_empty() {
            return Ok(());
        }

        //TODO: Add support for memory barriers to metal_rs so this can be more fine-grained
        // (use memoryBarrierWithScope)

        if let Some(render_encoder) = self.metal_render_encoder() {
            render_encoder.wait_for_fence(self.queue.metal_fence(), MTLRenderStages::Vertex);
        } else if let Some(compute_encoder) = self.metal_compute_encoder() {
            compute_encoder.wait_for_fence(self.queue.metal_fence());
        } else if let Some(blit_encoder) = self.metal_blit_encoder() {
            blit_encoder.wait_for_fence(self.queue.metal_fence());
        }

        self.queue.clear_barrier_flags();
        Ok(())
    }

    pub fn cmd_unbind_render_targets(&self) -> RafxResult<()> {
        // no action necessary
        Ok(())
    }

    pub fn cmd_set_viewport(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth_min: f32,
        depth_max: f32,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_set_scissor(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_set_stencil_reference_value(
        &self,
        value: u32,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_bind_pipeline(
        &self,
        pipeline: &RafxPipelineMetal,
    ) -> RafxResult<()> {
        let last_pipeline_type = self.last_pipeline_type.load(Ordering::Relaxed);
        self.last_pipeline_type.store(pipeline.pipeline_type() as u8, Ordering::Relaxed);

        let barrier_required = last_pipeline_type != pipeline.pipeline_type() as u8;

        match pipeline.pipeline_type() {
            RafxPipelineType::Graphics => {
                let render_encoder = self.metal_render_encoder().unwrap();
                render_encoder.set_render_pipeline_state(pipeline.metal_render_pipeline().unwrap());
                render_encoder.set_cull_mode(pipeline.mtl_cull_mode);
                render_encoder.set_triangle_fill_mode(pipeline.mtl_triangle_fill_mode);
                render_encoder.set_depth_bias(pipeline.mtl_depth_bias, pipeline.mtl_depth_bias_slope_scaled, 0.0);
                render_encoder.set_depth_clip_mode(pipeline.mtl_depth_clip_mode);
                if let Some(mtl_depth_stencil_state) = &pipeline.mtl_depth_stencil_state {
                    render_encoder.set_depth_stencil_state(mtl_depth_stencil_state);
                }

                self.primitive_type.store(mtl_primitve_type_to_u8(pipeline.mtl_primitive_type), Ordering::Relaxed);
                self.flush_render_targets_to_make_readable();
            }
            RafxPipelineType::Compute => {
                if !self.metal_compute_encoder().is_some() {
                    self.end_current_encoders(barrier_required);

                    let compute_encoder = self.metal_command_buffer().unwrap().new_compute_command_encoder();
                    self.swap_compute_encoder(Some(compute_encoder.to_owned()));
                }

                self.metal_compute_encoder().unwrap().set_compute_pipeline_state(pipeline.metal_compute_pipeline().unwrap());
                self.flush_render_targets_to_make_readable();
            }
        }

        Ok(())
    }

    fn flush_render_targets_to_make_readable(&self) {
        if let Some(render_encoder) = self.metal_render_encoder() {
            let mut guard = self.inner.lock().unwrap();
            for attachment in &guard.render_targets_to_make_readable {
                render_encoder.use_resource(attachment.texture().metal_texture().unwrap().metal_texture(), MTLResourceUsage::Read);
            }

            guard.render_targets_to_make_readable.clear();
        } else if let Some(compute_encoder) = self.metal_compute_encoder() {
            let mut guard = self.inner.lock().unwrap();
            for attachment in &guard.render_targets_to_make_readable {
                compute_encoder.use_resource(attachment.texture().metal_texture().unwrap().metal_texture(), MTLResourceUsage::Read);
            }

            guard.render_targets_to_make_readable.clear();
        }
    }

    pub fn cmd_bind_vertex_buffers(
        &self,
        first_binding: u32,
        bindings: &[RafxVertexBufferBinding],
    ) -> RafxResult<()> {
        let render_encoder = self.metal_render_encoder().unwrap();

        let mut binding_index = first_binding;
        for binding in bindings {
            render_encoder.set_vertex_buffer(
                binding_index as _,
                Some(binding.buffer.metal_buffer().unwrap().metal_buffer()),
                binding.offset as _
            );

            binding_index += 1;
        }

        Ok(())
    }

    pub fn cmd_bind_index_buffer(
        &self,
        binding: &RafxIndexBufferBinding,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_bind_descriptor_set(
        &self,
        descriptor_set_array: &RafxDescriptorSetArrayMetal,
        index: u32,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_bind_descriptor_set_handle(
        &self,
        root_signature: &RafxRootSignatureMetal,
        set_index: u32,
        descriptor_set_handle: &RafxDescriptorSetHandleMetal,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_draw(
        &self,
        vertex_count: u32,
        first_vertex: u32,
    ) -> RafxResult<()> {
        self.metal_render_encoder().unwrap().draw_primitives(
            self.primitive_type(),
            first_vertex as _,
            vertex_count as _
        );
        Ok(())
    }

    pub fn cmd_draw_instanced(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) -> RafxResult<()> {
        self.metal_render_encoder().unwrap().draw_primitives_instanced_base_instance(
            self.primitive_type(),
            first_vertex as _,
            vertex_count as _,
            instance_count as _,
            first_instance as _,
        );
        Ok(())
    }

    pub fn cmd_draw_indexed(
        &self,
        index_count: u32,
        first_index: u32,
        vertex_offset: i32,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_draw_indexed_instanced(
        &self,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_dispatch(
        &self,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_resource_barrier(
        &self,
        buffer_barriers: &[RafxBufferBarrier],
        texture_barriers: &[RafxTextureBarrier],
        render_target_barriers: &[RafxRenderTargetBarrier],
    ) -> RafxResult<()> {
        if !buffer_barriers.is_empty() {
            self.queue.add_barrier_flags(BarrierFlagsMetal::BUFFERS);
        }

        if !texture_barriers.is_empty() {
            self.queue.add_barrier_flags(BarrierFlagsMetal::TEXTURES);
        }

        if !render_target_barriers.is_empty() {
            self.queue.add_barrier_flags(BarrierFlagsMetal::RENDER_TARGETS);

            let mut guard = self.inner.lock().unwrap();
            for rt in render_target_barriers {
                if rt.src_state.intersects(RafxResourceState::RENDER_TARGET) &&
                    rt.dst_state.intersects(RafxResourceState::UNORDERED_ACCESS | RafxResourceState::SHADER_RESOURCE) {
                    guard.render_targets_to_make_readable.insert(rt.render_target.metal_render_target().unwrap().clone());
                }
            }
        }

        Ok(())
    }

    pub fn cmd_copy_buffer_to_buffer(
        &self,
        src_buffer: &RafxBufferMetal,
        dst_buffer: &RafxBufferMetal,
        src_offset: u64,
        dst_offset: u64,
        size: u64,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_copy_buffer_to_texture(
        &self,
        src_buffer: &RafxBufferMetal,
        dst_texture: &RafxTextureMetal,
        params: &RafxCmdCopyBufferToTextureParams,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_blit(
        &self,
        src_texture: &RafxTextureMetal,
        dst_texture: &RafxTextureMetal,
        params: &RafxCmdBlitParams,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub(crate) fn swap_command_buffer(&self, command_buffer: Option<metal_rs::CommandBuffer>) -> Option<metal_rs::CommandBuffer> {
        use foreign_types_shared::ForeignType;
        use foreign_types_shared::ForeignTypeRef;

        let ptr = if let Some(command_buffer) = command_buffer {
            command_buffer.as_ptr()
        } else {
            std::ptr::null_mut() as _
        };

        let ptr = self.command_buffer.swap(ptr, Ordering::Relaxed);
        if !ptr.is_null() {
            unsafe {
                Some(metal_rs::CommandBuffer::from_ptr(ptr))
            }
        } else {
            None
        }
    }

    fn swap_render_encoder(&self, render_encoder: Option<metal_rs::RenderCommandEncoder>) -> Option<metal_rs::RenderCommandEncoder> {
        use foreign_types_shared::ForeignType;
        use foreign_types_shared::ForeignTypeRef;

        let _has_render_encoder = render_encoder.is_some();
        let ptr = if let Some(render_encoder) = render_encoder {
            render_encoder.as_ptr()
        } else {
            std::ptr::null_mut() as _
        };

        let ptr = self.render_encoder.swap(ptr, Ordering::Relaxed);
        let old_encoder = if !ptr.is_null() {
            unsafe {
                Some(metal_rs::RenderCommandEncoder::from_ptr(ptr))
            }
        } else {
            None
        };

        // Verify only one encoder exists at a time
        debug_assert!(!_has_render_encoder || self.compute_encoder.load(Ordering::Relaxed).is_null());
        debug_assert!(!_has_render_encoder || self.blit_encoder.load(Ordering::Relaxed).is_null());
        old_encoder
    }

    fn swap_compute_encoder(&self, compute_encoder: Option<metal_rs::ComputeCommandEncoder>) -> Option<metal_rs::ComputeCommandEncoder> {
        use foreign_types_shared::ForeignType;
        use foreign_types_shared::ForeignTypeRef;

        let _has_compute_encoder = compute_encoder.is_some();
        let ptr = if let Some(compute_encoder) = compute_encoder {
            compute_encoder.as_ptr()
        } else {
            std::ptr::null_mut() as _
        };

        let ptr = self.compute_encoder.swap(ptr, Ordering::Relaxed);
        let old_encoder = if !ptr.is_null() {
            unsafe {
                Some(metal_rs::ComputeCommandEncoder::from_ptr(ptr))
            }
        } else {
            None
        };

        // Verify only one encoder exists at a time
        debug_assert!(!_has_compute_encoder || self.render_encoder.load(Ordering::Relaxed).is_null());
        debug_assert!(!_has_compute_encoder || self.blit_encoder.load(Ordering::Relaxed).is_null());
        old_encoder
    }

    fn swap_blit_encoder(&self, blit_encoder: Option<metal_rs::BlitCommandEncoder>) -> Option<metal_rs::BlitCommandEncoder> {
        use foreign_types_shared::ForeignType;
        use foreign_types_shared::ForeignTypeRef;

        let _has_blit_encoder = blit_encoder.is_some();
        let ptr = if let Some(blit_encoder) = blit_encoder {
            blit_encoder.as_ptr()
        } else {
            std::ptr::null_mut() as _
        };

        let ptr = self.blit_encoder.swap(ptr, Ordering::Relaxed);
        let old_encoder = if !ptr.is_null() {
            unsafe {
                Some(metal_rs::BlitCommandEncoder::from_ptr(ptr))
            }
        } else {
            None
        };

        // Verify only one encoder exists at a time
        debug_assert!(!_has_blit_encoder || self.render_encoder.load(Ordering::Relaxed).is_null());
        debug_assert!(!_has_blit_encoder || self.compute_encoder.load(Ordering::Relaxed).is_null());
        old_encoder
    }

    pub fn metal_command_buffer(&self) -> Option<&metal_rs::CommandBufferRef> {
        use foreign_types_shared::ForeignType;
        use foreign_types_shared::ForeignTypeRef;

        let command_buffer = self.command_buffer.load(Ordering::Relaxed);
        if !command_buffer.is_null() {
            unsafe {
                Some(metal_rs::CommandBufferRef::from_ptr(command_buffer))
            }
        } else {
            None
        }
    }

    pub fn metal_render_encoder(&self) -> Option<&metal_rs::RenderCommandEncoderRef> {
        use foreign_types_shared::ForeignType;
        use foreign_types_shared::ForeignTypeRef;

        let render_encoder = self.render_encoder.load(Ordering::Relaxed);
        if !render_encoder.is_null() {
            unsafe {
                Some(metal_rs::RenderCommandEncoderRef::from_ptr(render_encoder))
            }
        } else {
            None
        }
    }

    pub fn metal_compute_encoder(&self) -> Option<&metal_rs::ComputeCommandEncoderRef> {
        use foreign_types_shared::ForeignType;
        use foreign_types_shared::ForeignTypeRef;

        let compute_encoder = self.compute_encoder.load(Ordering::Relaxed);
        if !compute_encoder.is_null() {
            unsafe {
                Some(metal_rs::ComputeCommandEncoderRef::from_ptr(compute_encoder))
            }
        } else {
            None
        }
    }

    pub fn metal_blit_encoder(&self) -> Option<&metal_rs::BlitCommandEncoderRef> {
        use foreign_types_shared::ForeignType;
        use foreign_types_shared::ForeignTypeRef;

        let blit_encoder = self.blit_encoder.load(Ordering::Relaxed);
        if !blit_encoder.is_null() {
            unsafe {
                Some(metal_rs::BlitCommandEncoderRef::from_ptr(blit_encoder))
            }
        } else {
            None
        }
    }

    fn primitive_type(&self) -> MTLPrimitiveType {
        u8_to_mtl_primitve_type(self.primitive_type.load(Ordering::Relaxed))
    }
}

fn mtl_primitve_type_to_u8(primitive_type: MTLPrimitiveType) -> u8 {
    match primitive_type {
        MTLPrimitiveType::Point => 0,
        MTLPrimitiveType::Line => 1,
        MTLPrimitiveType::LineStrip => 2,
        MTLPrimitiveType::Triangle => 3,
        MTLPrimitiveType::TriangleStrip => 4,
    }
}

fn u8_to_mtl_primitve_type(primitive_type: u8) -> MTLPrimitiveType {
    match primitive_type {
        0 => MTLPrimitiveType::Point,
        1 => MTLPrimitiveType::Line,
        2 => MTLPrimitiveType::LineStrip,
        3 => MTLPrimitiveType::Triangle,
        4 => MTLPrimitiveType::TriangleStrip,
        _ => unreachable!()
    }
}