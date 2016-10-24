extern crate libc;
extern crate glsl_optimizer_sys as ffi;

use std::ffi::{CStr, CString};
use build::{Language, SourceKind};

pub struct Optimizer {
    ctx: *mut ffi::glslopt_ctx,
}

impl Optimizer {
    pub fn new(lang: Language) -> Optimizer {
        let lang = match lang {
            Language::OpenGl => ffi::kGlslTargetOpenGL,
            Language::OpenGlEs20 => ffi::kGlslTargetOpenGLES20,
            Language::OpenGlEs30 => ffi::kGlslTargetOpenGLES30,
        };
        Optimizer {
            ctx: unsafe { ffi::glslopt_initialize(lang) },
        }
    }

    pub fn optimize(&self, source: String, kind: SourceKind) -> Result<String, String> {
        if kind == SourceKind::Unknown {
            return Ok(source);
        }
        Shader::new(self, source, kind).source()
    }
}

impl Drop for Optimizer {
    fn drop(&mut self) {
        unsafe { ffi::glslopt_cleanup(self.ctx) };
    }
}

pub struct Shader {
    shader: *mut ffi::glslopt_shader,
}

impl Shader {
    fn new(opt: &Optimizer, source: String, kind: SourceKind) -> Shader {
        let kind = match kind {
            SourceKind::Vertex => ffi::kGlslOptShaderVertex,
            SourceKind::Fragment => ffi::kGlslOptShaderFragment,
            _ => unreachable!(),
        };
        let source = CString::new(source).unwrap();
        let shader = unsafe { ffi::glslopt_optimize(opt.ctx, kind, source.as_ptr(), ffi::kGlslOptionSkipPreprocessor) };
        Shader {
            shader: shader,
        }
    }

    fn source(&self) -> Result<String, String> {
        unsafe {
            if ffi::glslopt_get_status(self.shader) {
                Ok(CStr::from_ptr(ffi::glslopt_get_output(self.shader)).to_string_lossy().into_owned())
            } else {
                Err(CStr::from_ptr(ffi::glslopt_get_log(self.shader)).to_string_lossy().into_owned())
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { ffi::glslopt_shader_delete(self.shader) };
    }
}
