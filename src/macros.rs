#[macro_export]
macro_rules! counter {
    ($key:expr, $val:expr) => {
        $crate::counter!($key, $val,)
    };
    ($key:expr, $value:expr, $($tag_key:expr => $tag_val:expr),*) => {{
        $crate::record_metric($crate::Metric {
            key: &$key as &dyn ::core::fmt::Display,
            ty: $crate::MetricType::Counter,
            tags: &[$($crate::__metric_tag!($tag_key => $tag_val),)*],
            value: $crate::MetricValue {
                unit: $crate::MetricUnit::Unknown,
                value: &$value as &dyn ::core::fmt::Display,
            },
        });
    }};
}

#[macro_export]
macro_rules! __metric_tag {
    ($tag_key:expr => $tag_val:expr) => {
        (
            Some(&$tag_key as &dyn ::core::fmt::Display),
            &$tag_val as &dyn ::core::fmt::Display,
        )
    };
    ($tag_val:expr) => {
        (None, &$tag_val as &dyn ::core::fmt::Display)
    };
}
