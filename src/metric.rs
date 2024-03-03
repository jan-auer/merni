use crate::{MetricType, MetricUnit};

#[doc(hidden)]
pub struct Location<'a> {
    file: &'a str,
    line: u32,
    module_path: &'a str,
}

impl<'a> Location<'a> {
    pub const fn new(file: &'a str, line: u32, module_path: &'a str) -> Self {
        Self {
            file,
            line,
            module_path,
        }
    }
}

pub struct MetricMeta {
    ty: MetricType,
    unit: MetricUnit,
    key: &'static str,
    // TODO:
    // target: &'static str,
    location: Option<&'static Location<'static>>,
    pub(crate) tag_keys: &'static [&'static str],
}

pub struct TaggedMetric<const N: usize> {
    pub(crate) meta: MetricMeta,
}

impl MetricMeta {
    pub const fn new(ty: MetricType, unit: MetricUnit, key: &'static str) -> Self {
        Self {
            ty,
            unit,
            key,
            location: None,
            tag_keys: &[],
        }
    }

    pub const fn with_location(mut self, location: &'static Location<'static>) -> Self {
        self.location = Some(location);
        self
    }

    pub const fn with_tags<const N: usize>(
        mut self,
        tag_keys: &'static [&'static str; N],
    ) -> TaggedMetric<N> {
        self.tag_keys = tag_keys;
        TaggedMetric { meta: self }
    }

    pub fn ty(&self) -> MetricType {
        self.ty
    }

    pub fn unit(&self) -> MetricUnit {
        self.unit
    }

    pub fn key(&self) -> &'static str {
        self.key
    }

    pub fn file(&self) -> Option<&'static str> {
        self.location.as_ref().map(|l| l.file)
    }

    pub fn line(&self) -> Option<u32> {
        self.location.as_ref().map(|l| l.line)
    }

    pub fn module_path(&self) -> Option<&'static str> {
        self.location.as_ref().map(|l| l.module_path)
    }
}
