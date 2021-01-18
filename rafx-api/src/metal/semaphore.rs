use crate::metal::RafxDeviceContextMetal;
use crate::RafxResult;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct RafxSemaphoreMetal {
    device_context: RafxDeviceContextMetal,

    // Set to true when an operation is scheduled to signal this semaphore
    // Cleared when an operation is scheduled to consume this semaphore
    signal_available: AtomicBool,
}

impl RafxSemaphoreMetal {
    pub fn new(device_context: &RafxDeviceContextMetal) -> RafxResult<RafxSemaphoreMetal> {
        //TODO: Need to add support for new_event() in metal crate

        Ok(RafxSemaphoreMetal {
            device_context: device_context.clone(),
            //vk_semaphore,
            signal_available: AtomicBool::new(false),
        })
    }

    pub(crate) fn signal_available(&self) -> bool {
        self.signal_available.load(Ordering::Relaxed)
    }

    pub(crate) fn set_signal_available(
        &self,
        available: bool,
    ) {
        self.signal_available.store(available, Ordering::Relaxed);
    }
}
