use core::fmt::{self, Display};

#[non_exhaustive]
pub enum MetricType {
    Counter,
    Gauge,
    Distribution,
    // Timer,
    // Meter,
    // Histogram,
    // Set,
}

impl Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MetricType::Counter => "c",
            MetricType::Gauge => "g",
            MetricType::Distribution => "d",
        })
    }
}

#[non_exhaustive]
pub enum MetricUnit {
    Unknown,
}

#[non_exhaustive]
pub enum MetricValue<'a> {
    I64(i64),
    U64(u64),
    F64(f64),
    Str(&'a str),
}

impl Display for MetricValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricValue::I64(v) => v.fmt(f),
            MetricValue::U64(v) => v.fmt(f),
            MetricValue::F64(v) => v.fmt(f),
            MetricValue::Str(v) => v.fmt(f),
        }
    }
}

macro_rules! into_metric_value {
    ($($from:ident),+ => $variant:ident) => {
        $(
            impl From<$from> for MetricValue<'_> {
                #[inline(always)]
                fn from(f: $from) -> Self {
                    Self::$variant(f.into())
                }
            }
        )+
    };
}

into_metric_value!(i8, i16, i32, i64 => I64);
into_metric_value!(u8, u16, u32, u64 => U64);
into_metric_value!(f32, f64 => F64);

pub type MetricTags<'a> = &'a [(Option<&'a dyn Display>, &'a dyn Display)];

pub struct Metric<'a> {
    pub key: &'a dyn Display,
    pub ty: MetricType,
    pub unit: MetricUnit,

    pub tags: MetricTags<'a>,
    pub value: MetricValue<'a>,
}
