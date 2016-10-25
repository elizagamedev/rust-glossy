#[cfg(not(glossy_macros_only))]
#[macro_use]
extern crate lazy_static;
#[cfg(not(glossy_macros_only))]
pub mod build;
#[cfg(not(glossy_macros_only))]
pub use build::*;

/// Evaluates to a string of the glossy-processed shader source of the given filename.
#[macro_export]
macro_rules! shader {
    ($file:expr) => (include_str!(concat!(env!("OUT_DIR"), "/", $file)))
}

/// Returns the name of the shader file for the given __FILE__ value.
#[cfg(glossy_macros_only)]
pub fn file_id_to_name(id: u32) -> &'static str {
    include!(concat!(env!("OUT_DIR"), "/glossy_file_id_to_name.rs"))
}
