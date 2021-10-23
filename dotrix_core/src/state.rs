use std::any::Any;
use crate::ecs::{ Rule, StateId };

struct Entry {
    state_id: StateId,
    name: String,
    boxed: Box<dyn IntoState>,
}

/// Application States stack service
pub struct State {
    stack: Vec<Entry>,
    state_ptr: usize,
}

impl State {
    /// Sets pointer to data, holding information about the application state
    pub(crate) fn set_pointer(&mut self, state_ptr: usize) {
        self.state_ptr = state_ptr;
    }

    fn write_pointer(&self, value: StateId) {
        let state_ptr = self.state_ptr as *const StateId;
        if !state_ptr.is_null() {
            unsafe {
                *(self.state_ptr as *mut StateId) = value;
            }
        }
    }

    /// Returns a rule, so the system will run when the state is ON
    pub fn on<T: IntoState>() -> Rule {
        let state_id = StateId::of::<T>();
        Rule::StateOn(state_id)
    }

    /// Returns a rule, so the system will run when the state is OFF
    pub fn off<T: IntoState>() -> Rule {
        Rule::StateOff(StateId::of::<T>())
    }

    /// Pushes the application state to the stack
    pub fn push<T>(&mut self, state: T)
    where T: IntoState {
        let state_id = StateId::of::<T>();
        let name = String::from(std::any::type_name::<T>());
        self.stack.push(
            Entry {
                state_id,
                name,
                boxed: Box::new(state)
            }
        );
        self.write_pointer(state_id);
    }

    /// Pops the application state from the stack, and returns it
    pub fn pop<T: IntoState>(&mut self) -> Option<T> {
        self.pop_any().map(|boxed| {
            if boxed.is::<T>() {
                unsafe {
                    let raw: *mut dyn IntoState = Box::into_raw(boxed);
                    Some(*Box::from_raw(raw as *mut T))
                }
            } else {
                None
            }
        }).unwrap_or(None)
    }

    /// Pops the application state from the stack, but do not downcast it
    pub fn pop_any(&mut self) -> Option<Box<dyn IntoState>> {
        let last = self.stack.pop();
        let state_id = self.stack.last().map(|entry| entry.state_id)
            .unwrap_or_else(StateId::of::<bool>);
        self.write_pointer(state_id);
        last.map(|entry| entry.boxed)
    }

    /// Returns a referrence to the current state
    pub fn get<T: IntoState>(&mut self) -> Option<&T> {
        self.stack.last()
            .map(|entry| entry.boxed.downcast_ref())
            .unwrap_or(None)
    }

    /// Returns a mutable referrence to the current state
    pub fn get_mut<T: IntoState>(&mut self) -> Option<&mut T> {
        self.stack.last_mut()
            .map(|entry| entry.boxed.downcast_mut())
            .unwrap_or(None)
    }

    /// Returns [`StateId`] of current state
    pub fn id(&self) -> Option<StateId> {
        self.stack.last().map(|entry| entry.state_id)
    }

    /// Returns dump of current stack
    pub fn dump(&self) -> Vec<&str> {
        self.stack.iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            state_ptr: 0
        }
    }
}

/// Application state abstraction
pub trait IntoState: Any + Send + Sync + 'static {}
impl<T: 'static + Send + Sync> IntoState for T { }

impl dyn IntoState {

    /// Casts down the reference
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn IntoState as *const T)) }
        } else {
            None
        }
    }

    /// Casts down the mutual reference
    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn IntoState as *mut T)) }
        } else {
            None
        }
    }

    /// Checks if the reference is of specific type
    #[inline]
    fn is<T: Any>(&self) -> bool {
        std::any::TypeId::of::<T>() == self.type_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct SimpleState {}

    #[derive(Debug, Eq, PartialEq)]
    struct StateWithData(u32);

    #[derive(Debug, Eq, PartialEq)]
    struct StateWithAllocation(String);

    #[test]
    fn state_stack_and_downcasting() {
        let mut state = State::default();

        state.push(SimpleState {});
        state.push(StateWithData(123));
        state.push(StateWithAllocation(String::from("Allocated string")));

        let last: StateWithAllocation = state.pop().unwrap();
        assert_eq!(last, StateWithAllocation(String::from("Allocated string")));

        let last: StateWithData = state.pop().unwrap();
        assert_eq!(last, StateWithData(123));

        let last: SimpleState = state.pop().unwrap();
        assert_eq!(last, SimpleState {});
    }
}
