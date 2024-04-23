use smallvec::{InputTags, TagValues};
use smol_str::SmolStr;
use std::fmt::{Display, Write};
use std::hint::black_box;

// #[global_allocator]
// static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

#[divan::bench]
fn record_naive() {
    run(naive::record_tags)
}

#[divan::bench]
fn record_thread_local() {
    run(thread_local::record_tags)
}

#[divan::bench]
fn record_smallvec() {
    run(smallvec::record_tags)
}

fn run<F, R>(f: F)
where
    F: Fn(InputTags) -> R,
{
    let tags: &[&dyn Display] = &[&true];
    black_box(f(black_box(tags)));

    let tags: &[&dyn Display] = &[&123, &456];
    black_box(f(black_box(tags)));

    let tags: &[&dyn Display] = &[&"some more", &"and a bit longer", &"tag", &"values"];
    black_box(f(black_box(tags)));
}

#[path = "../src/tags.rs"]
mod smallvec;

mod naive {
    use super::*;

    pub fn record_tags(tags: InputTags) -> Option<Vec<String>> {
        if tags.is_empty() {
            return None;
        }

        Some(tags.iter().map(|d| d.to_string()).collect())
    }
}

mod thread_local {
    use std::cell::Cell;

    use super::*;

    thread_local! {
        static BUFFERS: Cell<(String, Vec<SmolStr>)> = const { Cell::new((String::new(), Vec::new())) };
    }

    pub fn record_tags(tags: InputTags) -> TagValues {
        if tags.is_empty() {
            return None;
        }

        let (mut string_buf, mut collected_tags) = BUFFERS.take();
        for tag in tags {
            write!(&mut string_buf, "{tag}").unwrap();
            collected_tags.push(SmolStr::new(string_buf.as_str()));
            string_buf.clear();
        }

        let tags: Box<[_]> = Box::from(collected_tags.as_slice());

        BUFFERS.set((string_buf, collected_tags));

        Some(tags)
    }
}
