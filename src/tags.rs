use std::fmt::{Display, Write};

use smallvec::SmallVec;
use smol_buf::Str24;

pub type InputTags<'a> = &'a [&'a dyn Display];
pub type TagValues = Option<Box<[Str24]>>;

pub fn record_tags(tags: InputTags) -> TagValues {
    if tags.is_empty() {
        return None;
    }

    let mut string_buf = StringBuf::<128>::default();
    let collected_tags = tags
        .iter()
        .map(|tag| {
            string_buf.clear();
            write!(&mut string_buf, "{tag}").unwrap();
            Str24::new(string_buf.as_str())
        })
        .collect();
    Some(collected_tags)
}

#[derive(Default)]
pub struct StringBuf<const N: usize> {
    buf: SmallVec<u8, N>,
}

impl<const N: usize> StringBuf<N> {
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.buf) }
    }
    pub fn clear(&mut self) {
        self.buf.clear()
    }
}

impl<const N: usize> Write for StringBuf<N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.extend_from_slice(s.as_bytes());
        Ok(())
    }
}
