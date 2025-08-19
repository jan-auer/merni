/// Create a [`MetricMeta`](crate::MetricMeta).
///
/// Instead of creating and emitting metrics manually, it is recommended to emit
/// metrics using the [`counter!`], [`gauge!`] or [`distribution!`] macros.
#[macro_export]
macro_rules! declare_metric {
    ($ty:ident => $key:literal $(@ $unit:ident)? : $($tag_key:literal),*) => {{
        const N: usize = $crate::macros::__count_helper([$($crate::__replace_expr!($tag_key ())),*]);
        static METRIC: $crate::TaggedMetricMeta<N> = $crate::MetricMeta::new(
            $crate::MetricType::$ty,
            $crate::__metric_unit!($($unit)?),
            $key
        )
        .with_tags(&[$($tag_key,)?]);
        &METRIC
    }};
    ($ty:ident => $key:literal $(@ $unit:ident)?) => {{
        static METRIC: $crate::MetricMeta = $crate::MetricMeta::new(
            $crate::MetricType::$ty,
            $crate::__metric_unit!($($unit)?),
            $key
        );
        &METRIC
    }};
}

/// Emits a counter metric with the current [`Dispatcher`](crate::Dispatcher).
#[macro_export]
macro_rules! counter {
    ($($tt:tt)+) => {
        $crate::__emit_metric!(Counter => $($tt)+);
    }
}

/// Emits a gauge metric with the current [`Dispatcher`](crate::Dispatcher).
#[macro_export]
macro_rules! gauge {
    ($($tt:tt)+) => {
        $crate::__emit_metric!(Gauge => $($tt)+);
    }
}

/// Emits a distribution metric with the current [`Dispatcher`](crate::Dispatcher).
#[macro_export]
macro_rules! distribution {
    ($($tt:tt)+) => {
        $crate::__emit_metric!(Distribution => $($tt)+);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __emit_metric {
    (
        $ty:ident => $key:literal $(@ $unit:ident)? : $value:expr
        , $($tag_key:literal => $tag_value:expr),+
    ) => {{
        $crate::with_dispatcher(|dispatcher| {
            let metric = $crate::declare_metric!(
                $ty => $key $(@ $unit)? :
                $($tag_key),+
            );
            dispatcher.emit_tagged(metric, $value, [$(&($tag_value)),+]);
        });
    }};
    ($ty:ident => $key:literal $(@ $unit:ident)? : $value:expr) => {{
        $crate::with_dispatcher(|dispatcher| {
            let metric = $crate::declare_metric!($ty => $key $(@ $unit)?);
            dispatcher.emit(metric, $value);
        });
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __metric_unit {
    () => {
        $crate::MetricUnit::Unknown
    };
    (s) => {
        $crate::MetricUnit::Seconds
    };
}

// These are taken from <https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#array-length>

#[doc(hidden)]
pub const fn __count_helper<const N: usize>(_: [(); N]) -> usize {
    N
}

#[macro_export]
#[doc(hidden)]
macro_rules! __replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}
