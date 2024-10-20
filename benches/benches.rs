pub mod metric {
    use merni::{counter, distribution, gauge};

    pub fn emit_simple() {
        counter!("some.counter": 1);
        counter!("some.tagged.counter": 2, "tag_key" => "tag_value");
        gauge!("some.gauge": 3);
        gauge!("some.tagged.gauge": 4, "tag_key" => "tag_value");
    }

    pub fn emit_distribution() {
        distribution!("some.distribution": 1);
        distribution!("some.tagged.distribution": 2, "tag_key" => "tag_value");
    }
}

pub mod tags {
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
        use std::cell::RefCell;

        thread_local! {
            static BUFFER: RefCell<String> = const { RefCell::new(String::new()) };
        }

        run(|tags| -> Option<Box<[Box<str>]>> {
            if tags.is_empty() {
                return None;
            }

            let collected_tags: Vec<_> = BUFFER.with_borrow_mut(|string_buf| {
                tags.iter()
                    .map(|d| {
                        string_buf.clear();
                        write!(string_buf, "{d}").unwrap();
                        string_buf.as_str().into()
                    })
                    .collect()
            });

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

    pub fn smolbuf() {
        run(|tags| -> Option<Box<[Str24]>> {
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
}
