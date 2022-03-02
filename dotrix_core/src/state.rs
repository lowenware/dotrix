use crate::id::OfType;
use crate::Id;
use std::any::Any;

/// Application state when system should run
#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub enum Rule {
    /// System runs ar any state
    Always,
    /// System runs at specific state
    StateOn(Id<State>),
    /// System does not run at specific state
    StateOff(Id<State>),
}

struct Entry {
    state_id: Id<State>,
    name: String,
    boxed: Box<dyn IntoState>,
}

/// Initial Meta state
///
/// Not in stack by default. Used for default value of internal state pointer
struct Meta;

/// Application States stack service
pub struct State {
    stack: Vec<Entry>,
    state_ptr: *const Id<State>,
}

// Secured by state_ptr controls
unsafe impl Send for State {}
unsafe impl Sync for State {}

impl State {
    /// Sets pointer to data, holding information about the application state
    pub(crate) fn set_pointer(&mut self, state_ptr: *const Id<State>) {
        self.state_ptr = state_ptr;
    }

    fn write_pointer(&self, value: Id<State>) {
        if !self.state_ptr.is_null() {
            unsafe {
                *(self.state_ptr as *mut Id<State>) = value;
            }
        }
    }

    /// Returns a rule, so the system will run when the state is ON
    pub fn on<T: IntoState>() -> Rule {
        let state_id: Id<State> = Id::of::<T>();
        Rule::StateOn(state_id)
    }

    /// Returns a rule, so the system will run when the state is OFF
    pub fn off<T: IntoState>() -> Rule {
        Rule::StateOff(Id::of::<T>())
    }

    /// Pushes the application state to the stack
    pub fn push<T>(&mut self, state: T)
    where
        T: IntoState,
    {
        let state_id: Id<State> = Id::of::<T>();
        let name = String::from(std::any::type_name::<T>());
        self.stack.push(Entry {
            state_id,
            name,
            boxed: Box::new(state),
        });
        self.write_pointer(state_id);
    }

    /// Pops the application state from the stack, and returns it
    pub fn pop<T: IntoState>(&mut self) -> Option<T> {
        self.pop_any()
            .map(|boxed| {
                if boxed.is::<T>() {
                    unsafe {
                        let raw: *mut dyn IntoState = Box::into_raw(boxed);
                        Some(*Box::from_raw(raw as *mut T))
                    }
                } else {
                    None
                }
            })
            .unwrap_or(None)
    }

    /// Pops the application state from the stack, but do not downcast it
    pub fn pop_any(&mut self) -> Option<Box<dyn IntoState>> {
        let last = self.stack.pop();
        let state_id = self
            .stack
            .last()
            .map(|entry| entry.state_id)
            .unwrap_or_else(Self::meta);
        self.write_pointer(state_id);
        last.map(|entry| entry.boxed)
    }

    /// Returns a referrence to the current state
    pub fn get<T: IntoState>(&self) -> Option<&T> {
        self.stack
            .last()
            .map(|entry| entry.boxed.downcast_ref())
            .unwrap_or(None)
    }

    /// Returns a mutable referrence to the current state
    pub fn get_mut<T: IntoState>(&mut self) -> Option<&mut T> {
        self.stack
            .last_mut()
            .map(|entry| entry.boxed.downcast_mut())
            .unwrap_or(None)
    }

    /// Returns [`Id<State>`] of current state
    pub fn id(&self) -> Option<Id<State>> {
        self.stack.last().map(|entry| entry.state_id)
    }

    /// Returns dump of current stack
    pub fn dump(&self) -> Vec<&str> {
        self.stack
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>()
    }

    /// Clears states stack
    pub fn clear(&mut self) {
        self.stack.clear();
        self.write_pointer(Self::meta());
    }

    /// Get id of meta state (one defines the state with empty stack)
    pub fn meta() -> Id<State> {
        Id::of::<Meta>()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            stack: vec![],
            state_ptr: std::ptr::null(),
        }
    }
}

impl OfType for Id<State> {
    fn of<T: std::any::Any>() -> Self {
        Id::from(std::any::TypeId::of::<T>())
    }
}

/// Application state abstraction
pub trait IntoState: Any + Send + Sync + 'static {}
impl<T: 'static + Send + Sync> IntoState for T {}

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

    #[test]
    fn state_clear() {
        let current_state: Id<State> = State::meta();
        let mut state = State::default();
        state.set_pointer(&current_state);

        let state_id = unsafe { *(state.state_ptr as *const Id<State>) };

        assert_eq!(state_id, Id::of::<Meta>());

        state.push(SimpleState {});
        state.push(StateWithData(123));
        state.push(StateWithAllocation(String::from("Allocated string")));

        state.clear();
        let state_id = unsafe { *(state.state_ptr as *const Id<State>) };
        assert_eq!(state_id, Id::of::<Meta>());
    }
}
