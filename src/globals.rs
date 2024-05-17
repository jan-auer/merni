use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use crate::Dispatcher;

static GLOBAL_DISPATCHER: OnceLock<Dispatcher> = OnceLock::new();
// because accessing a static global is faster than a thread local
static LOCAL_COUNT: AtomicUsize = AtomicUsize::new(0);
thread_local! {
    static LOCAL_DISPATCHER: Cell<Option<Dispatcher>> = const { Cell::new(None) };
}

/// Initialize the global [`Dispatcher`].
///
/// This will register the given `dispatcher` as the single global [`Dispatcher`] instance.
///
/// This function can only be called once, and subsequent calls will return an
/// [`Err`] in case a global [`Dispatcher`] has already been initialized.
pub fn set_global_dispatcher(dispatcher: Dispatcher) -> Result<(), Dispatcher> {
    let mut result = Err(dispatcher);
    {
        let result = &mut result;
        let _ = GLOBAL_DISPATCHER.get_or_init(|| std::mem::replace(result, Ok(())).unwrap_err());
    }
    result
}

/// A Guard for the thread-locally set [`Dispatcher`].
///
/// Reverts the previously set [`Dispatcher`] on [`Drop`].
pub struct LocalDispatcherGuard {
    previous: Option<Dispatcher>,
}

impl LocalDispatcherGuard {
    /// Drops the guard, returning [`Dispatcher`] that was set.
    pub fn take(self) -> Dispatcher {
        let dispatcher = LOCAL_DISPATCHER.take();
        drop(self); // this will restore the previous, and adjust `LOCAL_COUNT`
        dispatcher.unwrap()
    }
}

impl Drop for LocalDispatcherGuard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        if previous.is_none() {
            LOCAL_COUNT.fetch_sub(1, Ordering::Relaxed);
        }
        LOCAL_DISPATCHER.set(previous);
    }
}

/// Sets the thread-local [`Dispatcher`].
///
/// Returns a [`LocalDispatcherGuard`] which will revert to the previously set
/// [`Dispatcher`] on [`Drop`].
pub fn set_local_dispatcher(dispatcher: Dispatcher) -> LocalDispatcherGuard {
    let previous = LOCAL_DISPATCHER.replace(Some(dispatcher));
    if previous.is_none() {
        LOCAL_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    LocalDispatcherGuard { previous }
}

/// Runs the closure with a [`Dispatcher`] if one is configured.
///
/// This prefers the thread-local dispatcher if one is defined,
/// and otherwise falling back to the global dispatcher.
pub fn with_dispatcher<F, R>(f: F) -> R
where
    F: FnOnce(&Dispatcher) -> R,
    R: Default,
{
    if LOCAL_COUNT.load(Ordering::Relaxed) > 0 {
        if let Some(dispatcher) = LOCAL_DISPATCHER.take() {
            let result = f(&dispatcher);
            LOCAL_DISPATCHER.set(Some(dispatcher));
            return result;
        }
    }

    if let Some(dispatcher) = GLOBAL_DISPATCHER.get() {
        return f(dispatcher);
    }

    Default::default()
}
