//! Command handlers for the file-fairy CLI application

pub mod info;
pub mod organize;
pub mod registry;
pub mod scan;

pub use registry::{CommandHandler, CommandRegistry};
