use std::sync::Arc;
use crate::{RafxShaderModule, RafxShaderModuleDefMetal, RafxResult};
use crate::metal::RafxDeviceContextMetal;
use metal::MTLLanguageVersion;

#[derive(Debug)]
pub struct RafxShaderModuleMetalInner {
    library: metal::Library,
}

#[derive(Clone, Debug)]
pub struct RafxShaderModuleMetal {
    inner: Arc<RafxShaderModuleMetalInner>
}

impl RafxShaderModuleMetal {
    pub fn library(&self) -> &metal::LibraryRef {
        self.inner.library.as_ref()
    }

    pub fn new(
        device_context: &RafxDeviceContextMetal,
        data: RafxShaderModuleDefMetal,
    ) -> RafxResult<Self> {
        match data {
            RafxShaderModuleDefMetal::MetalLibBytes(bytes) => {
                RafxShaderModuleMetal::new_from_lib_bytes(device_context, bytes)
            }
            RafxShaderModuleDefMetal::MetalSrc(spv) => {
                RafxShaderModuleMetal::new_from_src(device_context, spv)
            }
        }
    }

    pub fn new_from_lib_bytes(
        device_context: &RafxDeviceContextMetal,
        data: &[u8],
    ) -> RafxResult<Self> {
        let library = device_context.device().new_library_with_data(data)?;

        let inner = RafxShaderModuleMetalInner {
            library
        };

        Ok(RafxShaderModuleMetal {
            inner: Arc::new(inner)
        })
    }

    pub fn new_from_src(
        device_context: &RafxDeviceContextMetal,
        src: &str,
    ) -> RafxResult<Self> {
        let mut compile_options = metal::CompileOptions::new();
        compile_options.set_language_version(MTLLanguageVersion::V2_1);
        let library = device_context.device().new_library_with_source(src, &compile_options)?;

        let inner = RafxShaderModuleMetalInner {
            library
        };

        Ok(RafxShaderModuleMetal {
            inner: Arc::new(inner)
        })
    }
}

impl Into<RafxShaderModule> for RafxShaderModuleMetal {
    fn into(self) -> RafxShaderModule {
        RafxShaderModule::Metal(self)
    }
}
