/// Create a [`Metric`](crate::Metric).
///
/// Instead of creating metrics directly, it is recommended to immediately record
/// metrics using the [`counter!`], [`gauge!`] or [`distribution!`] macros.
///
/// This is the recommended way to create a [`Metric`](crate::Metric), as the
/// implementation details of it might change.
#[macro_export]
macro_rules! declare_metric {
    ($ty:ident => $key:literal $(@ $unit:ident)? : $($tag_key:literal),*) => {{
        static LOCATION: $crate::Location = $crate::Location::new(file!(), line!(), module_path!());
        const N: usize = $crate::macros::__count_helper([$($crate::__replace_expr!($tag_key ())),*]);
        static META: $crate::TaggedMetric<N> = $crate::MetricMeta::new(
            $crate::MetricType::$ty,
            $crate::__metric_unit!($($unit)?),
            $key
        ).with_location(&LOCATION)
        .with_tags(&[$($tag_key,)?]);
        &META
    }};
    ($ty:ident => $key:literal $(@ $unit:ident)?) => {{
        static LOCATION: $crate::Location = $crate::Location::new(file!(), line!(), module_path!());
        static META: $crate::MetricMeta = $crate::MetricMeta::new(
            $crate::MetricType::$ty,
            $crate::__metric_unit!($($unit)?),
            $key
        ).with_location(&LOCATION);
        &META
    }};
}

// /// Records a counter [`Metric`](crate::Metric) with the global [`Recorder`](crate::Recorder).
// #[macro_export]
// macro_rules! counter {
//     ($($tt:tt)+) => {
//         $crate::__record_metric!(Counter: $($tt)+);
//     }
// }

// /// Records a gauge [`Metric`](crate::Metric) with the global [`Recorder`](crate::Recorder).
// #[macro_export]
// macro_rules! gauge {
//     ($($tt:tt)+) => {
//         $crate::__record_metric!(Gauge: $($tt)+);
//     }
// }

// /// Records a distribution [`Metric`](crate::Metric) with the global [`Recorder`](crate::Recorder).
// #[macro_export]
// macro_rules! distribution {
//     ($($tt:tt)+) => {
//         $crate::__record_metric!(Distribution: $($tt)+);
//     }
// }

// #[macro_export]
// #[doc(hidden)]
// macro_rules! __record_metric {
//     ($($tt:tt)+) => {{
//         $crate::record_metric($crate::metric!($($tt)+));
//     }};
// }

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
