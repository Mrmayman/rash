use std::collections::BTreeMap;

use rash_vm::data_types::ScratchObject;

use super::{
    error::{CompilerError, RegisterAction},
    structures::ThreadId,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum VariableIdentifier {
    Hash(String),
    TempVar {
        thread_id: ThreadId,
        register_id: RegisterId,
    },
}

#[derive(Clone, Copy)]
pub struct VMid(pub usize);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RegisterId(pub usize);

pub struct VariableAllocator {
    variables: BTreeMap<VariableIdentifier, (VMid, Option<ScratchObject>)>,
    registers_allocated: BTreeMap<RegisterId, bool>,
}

impl Default for VariableAllocator {
    fn default() -> Self {
        VariableAllocator::new()
    }
}

impl VariableAllocator {
    pub fn new() -> Self {
        Self {
            variables: BTreeMap::new(),
            registers_allocated: BTreeMap::new(),
        }
    }

    pub fn register_malloc(&mut self, thread_id: ThreadId) -> RegisterId {
        // Find an existing register that isn't allocated.
        match self
            .registers_allocated
            .iter_mut()
            .find(|(_, allocated)| !**allocated)
        {
            // If there's an existing unused register then allocate it.
            Some((id, allocated)) => {
                *allocated = true;
                *id
            }
            // Otherwise create a new register.
            None => {
                let alloc_len = self.registers_allocated.len();
                let vars_len = self.variables.len();

                self.variables.insert(
                    VariableIdentifier::TempVar {
                        thread_id,
                        register_id: RegisterId(alloc_len),
                    },
                    (VMid(vars_len), None),
                );

                self.registers_allocated.insert(RegisterId(alloc_len), true);

                RegisterId(alloc_len)
            }
        }
    }

    pub fn register_free(&mut self, id: RegisterId) -> Result<(), CompilerError> {
        let is_register_allocated = self
            .registers_allocated
            .get_mut(&id)
            .ok_or(CompilerError::RegisterNotFound(id, RegisterAction::Freeing))?;

        *is_register_allocated = false;

        Ok(())
    }

    pub fn register_get(&self, id: RegisterId, thread_id: ThreadId) -> Result<VMid, CompilerError> {
        self.variables
            .get(&VariableIdentifier::TempVar {
                thread_id,
                register_id: id,
            })
            .ok_or(CompilerError::RegisterNotFound(id, RegisterAction::Getting))
            .map(|id| id.0)
    }

    pub fn variable_add(&mut self, id: VariableIdentifier, data: Option<ScratchObject>) {
        let variables_len = self.variables.len();

        self.variables.insert(id, (VMid(variables_len), data));
    }

    pub fn variable_get(&self, id: &VariableIdentifier) -> Result<&VMid, CompilerError> {
        self.variables
            .get(id)
            .ok_or(CompilerError::VariableNotFoundWhenGetting(id.clone()))
            .map(|n| &n.0)
    }
}
