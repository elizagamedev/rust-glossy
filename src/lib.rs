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
include!(concat!(env!("OUT_DIR"), "/glossy_map.rs"));
