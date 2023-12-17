#[macro_export]
macro_rules! metric {
    ($ty:ident: $key:expr, $($unit:ident:)? $value:expr
        $(, $($tag_key:expr => $tag_val:expr),*)?
        $(; $($tag_only_val:expr),*)?
    ) => {{
        $crate::Metric {
            key: &$key,
            ty: $crate::MetricType::$ty,
            unit: $crate::__metric_unit!($($unit)?),

            tags: &[
                $($((Some(&$tag_key), &$tag_val),)*)?
                $($((None, &$tag_only_val),)*)?
            ],
            value: $value.into(),
        }
    }};
}

#[macro_export]
macro_rules! counter {
    ($($tt:tt)+) => {
        $crate::__record_metric!(Counter: $($tt)+);
    }
}

#[macro_export]
macro_rules! gauge {
    ($($tt:tt)+) => {
        $crate::__record_metric!(Gauge: $($tt)+);
    }
}

#[macro_export]
macro_rules! distribution {
    ($($tt:tt)+) => {
        $crate::__record_metric!(Distribution: $($tt)+);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __record_metric {
    ($($tt:tt)+) => {{
        $crate::record_metric($crate::metric!($($tt)+));
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
