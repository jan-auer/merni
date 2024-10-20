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
    let mut collected_tags = Box::<[Str24]>::new_uninit_slice(tags.len());

    for (i, tag) in tags.iter().enumerate() {
        write!(&mut string_buf, "{tag}").unwrap();
        unsafe {
            collected_tags[i]
                .as_mut_ptr()
                .write(Str24::new(string_buf.as_str()));
        }

        string_buf.clear();
    }

    Some(unsafe { collected_tags.assume_init() })
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
