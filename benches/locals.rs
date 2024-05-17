#![cfg_attr(feature = "nightly", feature(thread_local))]

use std::cell::{Cell, RefCell};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

fn main() {
    GLOBAL_USIZE.set(123).unwrap();
    GLOBAL_DROP.set(String::from("the global str")).unwrap();

    divan::main();
}

static GLOBAL_USIZE: OnceLock<usize> = OnceLock::new();

static LOCAL_COUNT: AtomicUsize = AtomicUsize::new(0);

static GLOBAL_DROP: OnceLock<String> = OnceLock::new();

mod _1_stable_copy {
    use super::*;

    thread_local! {
        static LOCAL_VALUE: Cell<Option<usize>> = const { Cell::new(None) };
    }

    fn set_local(update_counts: bool, value: usize) -> LocalGuard {
        let previous_value = LOCAL_VALUE.replace(Some(value));
        if update_counts && previous_value.is_none() {
            LOCAL_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        LocalGuard {
            update_counts,
            previous_value,
        }
    }

    struct LocalGuard {
        update_counts: bool,
        previous_value: Option<usize>,
    }

    impl Drop for LocalGuard {
        fn drop(&mut self) {
            let previous_value = self.previous_value.take();
            if self.update_counts && previous_value.is_none() {
                LOCAL_COUNT.fetch_sub(1, Ordering::Relaxed);
            }
            LOCAL_VALUE.set(previous_value);
        }
    }

    fn get_value_global_override() -> Option<usize> {
        if LOCAL_COUNT.load(Ordering::Relaxed) > 0 {
            if let Some(value) = black_box(LOCAL_VALUE.get()) {
                return Some(value);
            }
        }

        GLOBAL_USIZE.get().copied()
    }

    fn get_value_local_fallback() -> Option<usize> {
        if let Some(value) = black_box(LOCAL_VALUE.get()) {
            return Some(value);
        }

        GLOBAL_USIZE.get().copied()
    }

    #[divan::bench]
    fn _1_global_override_unset() {
        for _ in 0..1_000 {
            let value = black_box(get_value_global_override());
            assert_eq!(value, Some(123));
        }
    }

    #[divan::bench]
    fn _2_local_fallback_set() {
        let _guard = set_local(false, black_box(234));

        for _ in 0..1_000 {
            let value = black_box(get_value_local_fallback());
            assert_eq!(value, Some(234));
        }
    }

    #[divan::bench]
    fn _3_local_fallback_unset() {
        for _ in 0..1_000 {
            let value = black_box(get_value_local_fallback());
            assert_eq!(value, Some(123));
        }
    }

    #[divan::bench]
    fn _4_global_override_set() {
        let _guard = set_local(true, black_box(234));

        for _ in 0..1_000 {
            let value = black_box(get_value_global_override());
            assert_eq!(value, Some(234));
        }
    }
}

mod _2_stable_drop {
    use super::*;

    thread_local! {
        static LOCAL_VALUE: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    fn set_local(update_counts: bool, value: String) -> LocalGuard {
        let previous_value = LOCAL_VALUE.replace(Some(value));
        if update_counts && previous_value.is_none() {
            LOCAL_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        LocalGuard {
            update_counts,
            previous_value,
        }
    }

    struct LocalGuard {
        update_counts: bool,
        previous_value: Option<String>,
    }

    impl Drop for LocalGuard {
        fn drop(&mut self) {
            let previous_value = self.previous_value.take();
            if self.update_counts && previous_value.is_none() {
                LOCAL_COUNT.fetch_sub(1, Ordering::Relaxed);
            }
            LOCAL_VALUE.set(previous_value);
        }
    }

    fn with_value_global_override(f: impl FnOnce(Option<&str>)) {
        if LOCAL_COUNT.load(Ordering::Relaxed) > 0 {
            return LOCAL_VALUE.with_borrow(|value| {
                if let Some(value) = black_box(value) {
                    f(Some(value))
                } else {
                    f(GLOBAL_DROP.get().map(|s| s.as_str()))
                }
            });
        }
        f(GLOBAL_DROP.get().map(|s| s.as_str()))
    }

    fn with_value_local_fallback(f: impl FnOnce(Option<&str>)) {
        LOCAL_VALUE.with_borrow(|value| {
            if let Some(value) = black_box(value) {
                f(Some(value))
            } else {
                f(GLOBAL_DROP.get().map(|s| s.as_str()))
            }
        });
    }

    #[divan::bench]
    fn _1_global_override_unset() {
        for _ in 0..1_000 {
            with_value_global_override(|value| {
                assert_eq!(black_box(value), Some("the global str"));
            });
        }
    }

    #[divan::bench]
    fn _2_local_fallback_set() {
        let _guard = set_local(false, black_box(String::from("the local str")));

        for _ in 0..1_000 {
            with_value_local_fallback(|value| {
                assert_eq!(black_box(value), Some("the local str"));
            });
        }
    }

    #[divan::bench]
    fn _3_local_fallback_unset() {
        for _ in 0..1_000 {
            with_value_local_fallback(|value| {
                assert_eq!(black_box(value), Some("the global str"));
            });
        }
    }

    #[divan::bench]
    fn _4_global_override_set() {
        let _guard = set_local(true, black_box(String::from("the local str")));

        for _ in 0..1_000 {
            with_value_global_override(|value| {
                assert_eq!(black_box(value), Some("the local str"));
            });
        }
    }
}

#[cfg(feature = "nightly")]
mod _3_nightly_copy {
    use super::*;

    #[thread_local]
    static mut LOCAL_VALUE: Option<usize> = None;

    fn set_local(update_counts: bool, value: usize) -> LocalGuard {
        let previous_value = unsafe { LOCAL_VALUE.replace(value) };
        if update_counts && previous_value.is_none() {
            LOCAL_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        LocalGuard {
            update_counts,
            previous_value,
        }
    }

    struct LocalGuard {
        update_counts: bool,
        previous_value: Option<usize>,
    }

    impl Drop for LocalGuard {
        fn drop(&mut self) {
            let previous_value = self.previous_value.take();
            if self.update_counts && previous_value.is_none() {
                LOCAL_COUNT.fetch_sub(1, Ordering::Relaxed);
            }
            unsafe { LOCAL_VALUE = previous_value };
        }
    }

    fn get_value_global_override() -> Option<usize> {
        if LOCAL_COUNT.load(Ordering::Relaxed) > 0 {
            if let Some(value) = black_box(unsafe { LOCAL_VALUE }) {
                return Some(value);
            }
        }

        GLOBAL_USIZE.get().copied()
    }

    fn get_value_local_fallback() -> Option<usize> {
        if let Some(value) = black_box(unsafe { LOCAL_VALUE }) {
            return Some(value);
        }

        GLOBAL_USIZE.get().copied()
    }

    #[divan::bench]
    fn _1_global_override_unset() {
        for _ in 0..1_000 {
            let value = black_box(get_value_global_override());
            assert_eq!(value, Some(123));
        }
    }

    #[divan::bench]
    fn _2_local_fallback_set() {
        let _guard = set_local(false, black_box(234));

        for _ in 0..1_000 {
            let value = black_box(get_value_local_fallback());
            assert_eq!(value, Some(234));
        }
    }

    #[divan::bench]
    fn _3_local_fallback_unset() {
        for _ in 0..1_000 {
            let value = black_box(get_value_local_fallback());
            assert_eq!(value, Some(123));
        }
    }

    #[divan::bench]
    fn _4_global_override_set() {
        let _guard = set_local(true, black_box(234));

        for _ in 0..1_000 {
            let value = black_box(get_value_global_override());
            assert_eq!(value, Some(234));
        }
    }
}

#[cfg(feature = "nightly")]
mod _4_nightly_drop {
    use super::*;

    #[thread_local]
    static mut LOCAL_VALUE: Option<String> = None;

    fn set_local(update_counts: bool, value: String) -> LocalGuard {
        let previous_value = unsafe { LOCAL_VALUE.replace(value) };
        if update_counts && previous_value.is_none() {
            LOCAL_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        LocalGuard {
            update_counts,
            previous_value,
        }
    }

    struct LocalGuard {
        update_counts: bool,
        previous_value: Option<String>,
    }

    impl Drop for LocalGuard {
        fn drop(&mut self) {
            let previous_value = self.previous_value.take();
            if self.update_counts && previous_value.is_none() {
                LOCAL_COUNT.fetch_sub(1, Ordering::Relaxed);
            }
            unsafe { LOCAL_VALUE = previous_value };
        }
    }

    fn with_value_global_override(f: impl FnOnce(Option<&str>)) {
        if LOCAL_COUNT.load(Ordering::Relaxed) > 0 {
            if let Some(value) = black_box(unsafe { LOCAL_VALUE.as_ref() }) {
                return f(Some(value));
            }
        }
        f(GLOBAL_DROP.get().map(|s| s.as_str()))
    }

    fn with_value_local_fallback(f: impl FnOnce(Option<&str>)) {
        if let Some(value) = black_box(unsafe { LOCAL_VALUE.as_ref() }) {
            return f(Some(value));
        }
        f(GLOBAL_DROP.get().map(|s| s.as_str()))
    }

    #[divan::bench]
    fn _1_global_override_unset() {
        for _ in 0..1_000 {
            with_value_global_override(|value| {
                assert_eq!(black_box(value), Some("the global str"));
            });
        }
    }

    #[divan::bench]
    fn _2_local_fallback_set() {
        let _guard = set_local(false, black_box(String::from("the local str")));

        for _ in 0..1_000 {
            with_value_local_fallback(|value| {
                assert_eq!(black_box(value), Some("the local str"));
            });
        }
    }

    #[divan::bench]
    fn _3_local_fallback_unset() {
        for _ in 0..1_000 {
            with_value_local_fallback(|value| {
                assert_eq!(black_box(value), Some("the global str"));
            });
        }
    }

    #[divan::bench]
    fn _4_global_override_set() {
        let _guard = set_local(true, black_box(String::from("the local str")));

        for _ in 0..1_000 {
            with_value_global_override(|value| {
                assert_eq!(black_box(value), Some("the local str"));
            });
        }
    }
}
