//! Loads SB3 files as Scratch Projects into the Rash VM.
//!
//! This crate is responsible for:
//! - Extracting SB3 files,
//! - Loading the costumes and audio (TODO),
//! - Reading the blocks, variables and broadcasts
//!   and loading them into the Rash interpreter.
//!
//! # How to use
//! ```
//! // Reads the file into memory and parses it.
//! let mut project_file = rash_loader_sb3::ProjectFile::open("./test.sb3").unwrap();
//! // Compiles the blocks to instructions and loads the assets.
//! // Right now `vm: ()` because it's not finished.
//! let vm = project_file.load().unwrap();
//! ```
//! Currently [`ProjectFile::load`] does not return anything,
//! but when completed it will construct the Virtual Machine
//! and return it.

mod compiler;
pub mod error;
mod json_struct;
pub mod load;

pub use load::ProjectFile;
