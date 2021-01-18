use crate::{RafxCommandBufferDef, RafxResult, RafxColorRenderTargetBinding, RafxDepthRenderTargetBinding, RafxVertexBufferBinding, RafxBufferBarrier, RafxTextureBarrier, RafxRenderTargetBarrier, RafxCmdCopyBufferToTextureParams, RafxCmdBlitParams, RafxIndexBufferBinding};
use crate::metal::{RafxCommandPoolMetal, RafxPipelineMetal, RafxDescriptorSetArrayMetal, RafxRootSignatureMetal, RafxDescriptorSetHandleMetal, RafxBufferMetal, RafxTextureMetal, RafxQueueMetal};

#[derive(Debug)]
pub struct RafxCommandBufferMetal {
    queue: RafxQueueMetal
}

impl RafxCommandBufferMetal {
    pub fn new(
        command_pool: &RafxCommandPoolMetal,
        _command_buffer_def: &RafxCommandBufferDef,
    ) -> RafxResult<RafxCommandBufferMetal> {
        Ok(RafxCommandBufferMetal {
            queue: command_pool.queue().clone()
        })
    }

    pub fn begin(&self) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn end(&self) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn return_to_pool(&self) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_bind_render_targets(
        &self,
        color_targets: &[RafxColorRenderTargetBinding],
        depth_target: Option<RafxDepthRenderTargetBinding>,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn cmd_unbind_render_targets(&self) -> RafxResult<()> {
        unimplemented!();
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
        unimplemented!();
    }

    pub fn cmd_bind_vertex_buffers(
        &self,
        first_binding: u32,
        bindings: &[RafxVertexBufferBinding],
    ) -> RafxResult<()> {
        unimplemented!();
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
        unimplemented!();
    }

    pub fn cmd_draw_instanced(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) -> RafxResult<()> {
        unimplemented!();
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
        unimplemented!();
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
}
