pub mod bytecode {
    pub mod instructions;
    #[cfg(target_arch = "x86_64")]
    pub mod jit;
}
pub mod data_types;
pub mod vm_thread;
