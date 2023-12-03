use std::cell::RefCell;
use std::fmt::{self, Display};
use std::sync::OnceLock;

pub enum MetricType {
    Counter,
}

impl Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MetricType::Counter => "c",
        })
    }
}

pub enum MetricUnit {
    Unknown,
}

pub struct MetricValue<'a> {
    pub unit: MetricUnit,
    pub value: &'a dyn Display,
}

type MetricTags<'a> = &'a [(Option<&'a dyn Display>, &'a dyn Display)];

pub struct Metric<'a> {
    pub key: &'a dyn Display,
    pub ty: MetricType,
    pub tags: MetricTags<'a>,
    pub value: MetricValue<'a>,
}

pub trait Recorder {
    fn emit(&self, metric: &str);
}

impl<T: Recorder + ?Sized> Recorder for Box<T> {
    fn emit(&self, metric: &str) {
        (**self).emit(metric)
    }
}

static GLOBAL_RECORDER: OnceLock<Box<dyn Recorder + Send + Sync + 'static>> = OnceLock::new();

thread_local! {
    pub static STRING_BUFFER: RefCell<String> = const { RefCell::new(String::new()) };
}

pub fn init<R: Recorder + Send + Sync + 'static>(recorder: R) -> Result<(), R> {
    let mut result = Err(recorder);
    {
        let result = &mut result;
        let _ = GLOBAL_RECORDER.get_or_init(|| {
            let recorder = std::mem::replace(result, Ok(())).unwrap_err();
            Box::new(recorder)
        });
    }
    result
}

pub fn with_recorder<F: FnOnce(&dyn Recorder)>(f: F) {
    if let Some(recorder) = GLOBAL_RECORDER.get() {
        f(recorder)
    }
}

impl Metric<'_> {
    fn write_base_metric(&self, s: &mut String) {
        use fmt::Write;
        let _ = write!(s, "{}:{}|{}", self.key, self.value.value, self.ty);
    }

    fn write_tags(&self, s: &mut String) {
        use fmt::Write;
        if !self.tags.is_empty() {
            s.push_str("|#");
            for (i, &(key, value)) in self.tags.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                if let Some(key) = key {
                    let _ = write!(s, "{key}");
                    s.push(':');
                }
                let _ = write!(s, "{value}");
            }
        }
    }
}

pub fn record_metric(metric: Metric<'_>) {
    if let Some(recorder) = GLOBAL_RECORDER.get() {
        STRING_BUFFER.with_borrow_mut(|s| {
            s.clear();
            metric.write_base_metric(s);
            metric.write_tags(s);
            recorder.emit(s);
        });
    }
}

#[macro_export]
macro_rules! counter {
    ($key:expr, $val:expr) => {
        $crate::counter!($key, $val,)
    };
    ($key:expr, $value:expr, $($tag_key:expr => $tag_val:expr),*) => {{
        $crate::record_metric($crate::Metric {
            key: &$key as &dyn ::core::fmt::Display,
            ty: $crate::MetricType::Counter,
            tags: &[$($crate::_metric_tag!($tag_key => $tag_val),)*],
            value: $crate::MetricValue {
                unit: $crate::MetricUnit::Unknown,
                value: &$value as &dyn ::core::fmt::Display,
            },
        });
    }};
}

#[macro_export]
macro_rules! _metric_tag {
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

#[cfg(test)]
mod tests {
    // use super::*;

    //     #[test]
    //     fn test_metric() {
    //         use fmt::Write;
    //         let mut s = String::new();
    //         write!(
    //             &mut s,
    //             "{}",
    //             Metric {
    //                 key: format_args!("counter.{}", "dynamic"),
    //                 ty: MetricType::Counter,
    //                 tags: &[],
    //                 value: MetricValue {
    //                     unit: MetricUnit::Unknown,
    //                     value: 1,
    //                 },
    //             }
    //         )
    //         .unwrap();
    //         dbg!(s);
    //     }
}
