use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use crate::Dispatcher;

static GLOBAL_DISPATCHER: OnceLock<Dispatcher> = OnceLock::new();
// because accessing a static global is faster than a thread local
static LOCAL_COUNT: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static LOCAL_DISPATCHER: RefCell<Option<&Dispatcher>> = const { RefCell::new(None) };
}

/// Initialize the global [`Recorder`].
///
/// This will register the given `recorder` as the single global [`Recorder`] instance.
///
/// This function can only be called once, and subsequent calls will return an
/// [`Err`] in case a global [`Recorder`] has already been initialized.
pub fn set_global_default(dispatcher: Dispatcher) -> Result<(), Dispatcher> {
    let mut result = Err(dispatcher);
    {
        let result = &mut result;
        let _ = GLOBAL_DISPATCHER.get_or_init(|| std::mem::replace(result, Ok(())).unwrap_err());
    }
    result
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
    if LOCAL_COUNT.load(Ordering::Acquire) > 0 {
        if let Some(result) = LOCAL_DISPATCHER.with_borrow(|dispatcher| {
            if let Some(dispatcher) = dispatcher {
                return Some(f(dispatcher));
            }
            None
        }) {
            return result;
        }
    }

    if let Some(dispatcher) = GLOBAL_DISPATCHER.get() {
        return f(dispatcher);
    }

    Default::default()
}
