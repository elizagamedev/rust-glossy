//! Glossy is a GLSL source loading crate for Rust which supports the `#include`
//! directive and shader optimization at compile time via
//! [glsl-optimizer](https://github.com/aras-p/glsl-optimizer).
//!
//! Refer to the [GitHub repository](https://github.com/mathewv/rust-glossy) for more information.
//!
//! # Example Usage
//!
//! In build script `build.rs`:
//!
//! ```
//! extern crate glossy_codegen as glsl;
//!
//! void main() {
//!    glsl::Config::new(glsl::Language::OpenGl)
//!        .vertex("shaders/*.vert")
//!        .fragment("shaders/*.frag")
//!        .include("shaders/include/*")
//!        .optimize()
//!        .build();
//! }
//! ```
//!
//! In Rust source file `main.rs`:
//!
//! ```
//! #[macro_use]
//! extern crate glossy;
//! extern crate glium;
//!
//! void main() {
//!     // ...
//!     glium::Program::from_source(gl, shader!("sprite.vert"), shader!("sprite.frag"), None)
//!         .unwrap();
//!     // ...
//! }
//! ```
//!
//! In shader source file `shader.frag`:
//!
//! ```glsl
//! #version 120
//! #include "common.glsl"
//!
//! void main() {
//!     float v = common_func(common_uniform);
//!     // ...
//! }
//! ```

/// Evaluates to a string of the glossy-processed shader source of the given filename.
#[macro_export]
macro_rules! shader {
    ($file:expr) => (include_str!(concat!(env!("OUT_DIR"), "/", $file)))
}

/// Returns the name of the shader file for the given __FILE__ value.
#[macro_export]
macro_rules! shader_id_to_name {
    ($id:expr) => {
        include!(concat!(env!("OUT_DIR"), "/glossy_file_id_to_name.rs"))($id)
    }
}
