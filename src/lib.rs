pub mod charset;
pub mod core;
pub mod font;
pub mod project;
pub mod shader_gpu;
pub mod shaders;
pub mod typeface;

pub mod guides;
pub mod settings;

/// Public release version and independent internal build identifier.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_ID: &str = include_str!("../BUILD_ID");
