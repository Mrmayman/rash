pub mod bytecode {
    pub mod instructions;
}
pub mod data_types;
pub mod vm_thread;

#[cfg(feature = "jit")]
pub mod jit;
