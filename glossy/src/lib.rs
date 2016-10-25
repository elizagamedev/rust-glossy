/// Evaluates to a string of the glossy-processed shader source of the given filename.
#[macro_export]
macro_rules! shader {
    ($file:expr) => (include_str!(concat!(env!("OUT_DIR"), "/", $file)))
}

/// Returns the name of the shader file for the given __FILE__ value.
#[macro_export]
macro_rules! shader_id_to_name {
    ($id:expr) => (include!(concat!(env!("OUT_DIR"), "/glossy_file_id_to_name.rs")))
}
