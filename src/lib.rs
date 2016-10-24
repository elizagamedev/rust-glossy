#[cfg(not(glossy_macros_only))]
#[macro_use]
extern crate lazy_static;
#[cfg(not(glossy_macros_only))]
pub mod build;
#[cfg(not(glossy_macros_only))]
pub use build::*;

/// Evaluates to a string of the glossy-processed shader source of the given filename.
#[macro_export]
macro_rules! glossy_shader {
    ($file:expr) => (include_str!(concat!(env!("OUT_DIR"), "/", $file)))
}

/// Evaluates to a HashMap which maps __FILE__ IDs to include file names.
#[macro_export]
macro_rules! glossy_file_ids {
    () => (include!(concat!(env!("OUT_DIR"), "/glossy_map.rs")))
}
