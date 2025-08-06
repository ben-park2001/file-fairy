//! File organization module
//!
//! This module provides the core functionality for organizing files based on their content.
//! It's split into several submodules for better separation of concerns:
//!
//! - `action`: Defines the data model for organization actions
//! - `options`: Configuration options for organization operations
//! - `results`: Results and reporting for organization operations
//! - `service`: The main service that orchestrates file organization

pub mod action;
pub mod options;
pub mod results;
pub mod service;

pub use action::OrganizeAction;
pub use options::OrganizeOptions;
pub use results::OrganizeResults;
pub use service::FileOrganizeService;
