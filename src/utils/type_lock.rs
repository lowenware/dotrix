use std::any::TypeId;
use std::collections::HashMap;

/// Lock modes enumeration
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum LockMode {
    /// Data is locked for reading by number of consumers
    ReadOnly(u32),
    /// Data is locked for writing by some consumer
    ReadWrite,
}

/// Lock indicator
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Lock {
    /// Type is locked for reading only
    ReadOnly(TypeId),
    /// Type is locked for reading and writing
    ReadWrite(TypeId),
}

/// Type lock
#[derive(Default)]
pub struct TypeLock {
    data: HashMap<TypeId, LockMode>,
}

impl TypeLock {
    /// Constructs new instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a lock for provided types
    pub fn lock(&mut self, locks: &[Lock]) -> bool {
        for lock in locks.iter() {
            let can_lock = match lock {
                Lock::ReadOnly(type_id) => self
                    .data
                    .get(type_id)
                    .map(|mode| (*mode != LockMode::ReadWrite))
                    .unwrap_or(true),
                Lock::ReadWrite(type_id) => !self.data.contains_key(type_id),
            };
            if !can_lock {
                return false;
            }
        }

        for lock in locks.iter() {
            match lock {
                Lock::ReadOnly(type_id) => {
                    if let LockMode::ReadOnly(refs) =
                        self.data.entry(*type_id).or_insert(LockMode::ReadOnly(0))
                    {
                        *refs += 1;
                    }
                }
                Lock::ReadWrite(type_id) => {
                    self.data.insert(*type_id, LockMode::ReadWrite);
                }
            }
        }
        true
    }

    /// Releases lock for provided types
    pub fn unlock(&mut self, locks: &[Lock]) {
        for lock in locks.iter() {
            let mut remove_type_id = None;
            match lock {
                Lock::ReadOnly(type_id) => {
                    if let Some(LockMode::ReadOnly(refs)) = self.data.get_mut(type_id) {
                        if *refs == 1 {
                            remove_type_id = Some(*type_id);
                        } else {
                            *refs -= 1;
                        }
                    } else {
                        panic!("Unlock in ReadOnly mode has failed");
                    }
                }
                Lock::ReadWrite(type_id) => {
                    if let Some(LockMode::ReadWrite) = self.data.get(type_id) {
                        remove_type_id = Some(*type_id);
                    } else {
                        panic!("Unlock in ReadWrite mode has failed");
                    }
                }
            };

            if let Some(type_id) = remove_type_id.as_ref() {
                self.data.remove(type_id);
            }
        }
    }
}
