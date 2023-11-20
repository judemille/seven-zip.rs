#![warn(clippy::all, clippy::pedantic)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("This library does not support 32-bit systems. If you have encountered this error due to having a higher than 64-bit pointer width, congratulations! Reach out to me.");

pub mod error;
pub(crate) mod header;
pub mod reader;
pub mod types;
