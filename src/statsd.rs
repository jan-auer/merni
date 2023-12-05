use std::cell::RefCell;
use std::fmt;

use crate::{Metric, Recorder};

thread_local! {
    static STRING_BUFFER: RefCell<String> = const { RefCell::new(String::new()) };
}

pub trait MetricSink {
    fn emit(&self, metric: &str);
}

pub struct StatsdRecorder<S> {
    prefix: String,
    sink: S,
    tags: Vec<(Option<String>, String)>,
}

impl<S> StatsdRecorder<S> {
    pub fn new(prefix: &str, sink: S) -> Self {
        let prefix = if prefix.is_empty() {
            String::new()
        } else {
            format!("{}.", prefix.trim_end_matches('.'))
        };
        Self {
            prefix,
            sink,
            tags: vec![],
        }
    }

    pub fn with_tag(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.tags.push((Some(key.to_string()), value.to_string()));
        self
    }

    pub fn with_tag_value(mut self, value: impl ToString) -> Self {
        self.tags.push((None, value.to_string()));
        self
    }
}

impl<S: MetricSink> Recorder for StatsdRecorder<S> {
    fn record_metric(&self, metric: Metric<'_>) {
        STRING_BUFFER.with_borrow_mut(|s| {
            s.clear();
            s.reserve(256);
            s.push_str(&self.prefix);
            metric.write_base_metric(s);
            metric.write_tags(&self.tags, s);
            self.sink.emit(s);
        });
    }
}

impl Metric<'_> {
    pub(crate) fn write_base_metric(&self, s: &mut String) {
        use fmt::Write;
        let _ = write!(s, "{}:{}|{}", self.key, self.value, self.ty);
    }

    pub(crate) fn write_tags(&self, global_tags: &[(Option<String>, String)], s: &mut String) {
        use fmt::Write;
        if !global_tags.is_empty() || !self.tags.is_empty() {
            s.push_str("|#");

            for (i, (key, value)) in global_tags.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                if let Some(key) = key {
                    let _ = write!(s, "{key}");
                    s.push(':');
                }
                let _ = write!(s, "{value}");
            }

            if !global_tags.is_empty() {
                s.push(',');
            }

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
