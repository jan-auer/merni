use core::fmt::{self, Display};

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

pub type MetricTags<'a> = &'a [(Option<&'a dyn Display>, &'a dyn Display)];

pub struct Metric<'a> {
    pub key: &'a dyn Display,
    pub ty: MetricType,
    pub tags: MetricTags<'a>,
    pub value: MetricValue<'a>,
}

impl Metric<'_> {
    pub(crate) fn write_base_metric(&self, s: &mut String) {
        use fmt::Write;
        let _ = write!(s, "{}:{}|{}", self.key, self.value.value, self.ty);
    }

    pub(crate) fn write_tags(&self, s: &mut String) {
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
