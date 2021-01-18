use crate::metal::{RafxDeviceContextMetal, RafxFenceMetal, RafxSemaphoreMetal, RafxSwapchainMetal, RafxCommandPoolMetal, RafxCommandBufferMetal};
use crate::{RafxQueueType, RafxDeviceContext, RafxResult, RafxCommandPoolDef, RafxPresentSuccessResult};
use std::sync::Arc;

#[derive(Debug)]
pub struct RafxQueueMetalInner {
    device_context: RafxDeviceContextMetal,
    queue_type: RafxQueueType,
    queue: metal::CommandQueue,
}

#[derive(Clone, Debug)]
pub struct RafxQueueMetal {
    inner: Arc<RafxQueueMetalInner>
}

impl RafxQueueMetal {
    pub fn queue_type(&self) -> RafxQueueType {
        self.inner.queue_type
    }

    pub fn device_context(&self) -> &RafxDeviceContextMetal {
        &self.inner.device_context
    }

    pub fn create_command_pool(
        &self,
        command_pool_def: &RafxCommandPoolDef,
    ) -> RafxResult<RafxCommandPoolMetal> {
        RafxCommandPoolMetal::new(&self, command_pool_def)
    }

    pub fn new(
        device_context: &RafxDeviceContextMetal,
        queue_type: RafxQueueType,
    ) -> RafxResult<RafxQueueMetal> {
        let queue = device_context.device().new_command_queue();

        let inner = RafxQueueMetalInner {
            device_context: device_context.clone(),
            queue_type,
            queue
        };

        Ok(RafxQueueMetal {
            inner: Arc::new(inner)
        })
    }

    pub fn wait_for_queue_idle(&self) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn submit(
        &self,
        command_buffers: &[&RafxCommandBufferMetal],
        wait_semaphores: &[&RafxSemaphoreMetal],
        signal_semaphores: &[&RafxSemaphoreMetal],
        signal_fence: Option<&RafxFenceMetal>,
    ) -> RafxResult<()> {
        unimplemented!();
    }

    pub fn present(
        &self,
        swapchain: &RafxSwapchainMetal,
        wait_semaphores: &[&RafxSemaphoreMetal],
        image_index: u32,
    ) -> RafxResult<RafxPresentSuccessResult> {
        unimplemented!();
    }
}