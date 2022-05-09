//! When an asset changes it may need to reload
//! and rebind to the pipeline. This Trait is used
//! for signalling such a reload.
//!

use std::time::Instant;

#[derive(Debug, Clone, Copy)]
/// Denotes the kinds of reload/changes of data that can occur
pub enum ReloadKind {
    /// No change occured in the last load
    NoChange,
    /// Data was updated but a rebind was not required
    Update,
    /// Data was updated in a way that requires a rebind
    Reload,
}

#[derive(Debug)]
/// Used to track the state of reloads
pub struct ReloadState {
    /// The last instant that an update occured
    ///
    /// This is used to signal that update is required
    /// in assets that are not processed every frame
    pub last_update_at: Instant,
    /// The last instant that an reload occured
    ///
    /// This is used to signal that reload is required
    /// in assets that are not processed every frame
    pub last_reload_at: Instant,
}

impl Default for ReloadState {
    fn default() -> Self {
        Self {
            last_update_at: Instant::now(),
            last_reload_at: Instant::now(),
        }
    }
}

/// Describes an object that has gpu represntable data
/// that may update and needs periodic (re-)loading
///
/// A function that needs to for check changes since an Instance
/// should call [`changed_since`]
pub trait Reloadable {
    /// Get a ref to the `[ReloadState]` that holds relevent reload
    /// data
    fn get_reload_state(&self) -> &ReloadState;

    /// Get the kind of changes that has occured since a certain cycle
    fn changes_since(&self, since_then: Instant) -> ReloadKind {
        let reload_state = self.get_reload_state();

        if reload_state.last_reload_at >= since_then {
            ReloadKind::Reload
        } else if reload_state.last_update_at >= since_then {
            ReloadKind::Update
        } else {
            ReloadKind::NoChange
        }
    }
}

/// Describes an object that has gpu represntable data
/// that may update and needs periodic (re-)loading
///
/// A loading function should call `[flag_updated]` and `[flag_reload]`
/// depending on if an update occured or if a reload is required
pub trait ReloadableMut: Reloadable {
    /// Flag as having data updated but that no rebinding is required
    /// as happening this frame
    ///
    /// Usually this means that the data has changed but the amount/format
    /// of the data has not changed
    fn flag_update(&mut self) {
        self.get_reload_state_mut().last_update_at = Instant::now();
    }

    /// Flag as having data updated and that it require a rebind
    fn flag_reload(&mut self) {
        self.get_reload_state_mut().last_reload_at = Instant::now();
    }

    /// Get a mutable ref to the `[ReloadState]` that holds relevent reload
    /// data
    fn get_reload_state_mut(&mut self) -> &mut ReloadState;
}
