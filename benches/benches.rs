use std::fmt::Write;

use smol_buf::Str24;
use smol_str::{format_smolstr, SmolStr};

pub fn vec_string() {
    run(|tags| -> Vec<String> { tags.iter().map(|d| d.to_string()).collect() })
}

pub fn boxed_string() {
    run(|tags| -> Option<Box<[String]>> {
        if tags.is_empty() {
            return None;
        }

        let collected_tags: Vec<_> = tags.iter().map(|d| d.to_string()).collect();
        Some(collected_tags.into_boxed_slice())
    })
}

pub fn boxed_boxed() {
    run(|tags| -> Option<Box<[Box<str>]>> {
        if tags.is_empty() {
            return None;
        }

        let collected_tags: Vec<_> = tags
            .iter()
            .map(|d| d.to_string().into_boxed_str())
            .collect();
        Some(collected_tags.into_boxed_slice())
    })
}

pub fn thread_local() {
    use std::cell::Cell;

    thread_local! {
        static BUFFER: Cell<String> = const { Cell::new(String::new()) };
    }

    run(|tags| -> Option<Box<[Box<str>]>> {
        if tags.is_empty() {
            return None;
        }

        let mut string_buf = BUFFER.take();

        let collected_tags: Vec<_> = tags
            .iter()
            .map(|d| {
                string_buf.clear();
                write!(&mut string_buf, "{d}").unwrap();
                string_buf.as_str().into()
            })
            .collect();

        BUFFER.set(string_buf);

        Some(collected_tags.into_boxed_slice())
    })
}

pub fn smallvec() {
    run(|tags| -> Option<Box<[Box<str>]>> {
        if tags.is_empty() {
            return None;
        }

        let mut string_buf = StringBuf::<128>::default();

        let collected_tags: Vec<_> = tags
            .iter()
            .map(|d| {
                string_buf.clear();
                write!(&mut string_buf, "{d}").unwrap();
                string_buf.as_str().into()
            })
            .collect();
        Some(collected_tags.into_boxed_slice())
    })
}

pub fn smolstr() {
    run(|tags| -> Option<Box<[SmolStr]>> {
        if tags.is_empty() {
            return None;
        }
        let collected_tags: Vec<_> = tags.iter().map(|d| format_smolstr!("{d}")).collect();
        Some(collected_tags.into_boxed_slice())
    })
}

pub fn smallvec_smolstr() {
    run(|tags| -> Option<Box<[SmolStr]>> {
        if tags.is_empty() {
            return None;
        }

        let mut string_buf = StringBuf::<128>::default();

        let collected_tags: Vec<_> = tags
            .iter()
            .map(|d| {
                string_buf.clear();
                write!(&mut string_buf, "{d}").unwrap();
                string_buf.as_str().into()
            })
            .collect();
        Some(collected_tags.into_boxed_slice())
    })
}

pub fn smolbuf_opt() {
    run(|tags| -> Option<Box<[Str24]>> {
        if tags.is_empty() {
            return None;
        }

        let mut string_buf = StringBuf::<128>::default();
        let mut collected_tags = Vec::with_capacity(tags.len());
        for tag in tags {
            string_buf.clear();
            write!(&mut string_buf, "{tag}").unwrap();
            collected_tags.push(Str24::new(string_buf.as_str()));
        }
        Some(collected_tags.into_boxed_slice())
    })
}

type InputTags<'a> = &'a [&'a dyn std::fmt::Display];

fn run<F, R>(f: F)
where
    F: Fn(InputTags) -> R,
{
    use std::hint::black_box;

    let tags: InputTags = &[];
    black_box(f(black_box(tags)));

    let tags: InputTags = &[&true];
    black_box(f(black_box(tags)));

    let tags: InputTags = &[&123u32, &456u64];
    black_box(f(black_box(tags)));

    let tags: InputTags = &[&123f32, &b'b', &'c'];
    black_box(f(black_box(tags)));

    let tags: InputTags = &[
        &"some more, ",
        &"and a bit longer",
        &"tag values.",
        &"with one a bit longer than the capacity of a smol_str",
    ];
    black_box(f(black_box(tags)));
}

#[derive(Default)]
struct StringBuf<const N: usize> {
    buf: smallvec::SmallVec<u8, N>,
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
