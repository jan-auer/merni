# Mérni

> Hungarian: to measure
> [wiktionary](https://en.wiktionary.org/wiki/m%C3%A9r#Hungarian)

An opinionated metrics crate with low overhead, compatible with [`cadence`].

## Usage

To start recording metrics, one has to create a new [`StatsdRecorder`],
and register it as the global recorder using [`init`].
The recorder passes `statsd`-formatted metrics to a [`MetricSink`].
When the `cadence1` feature is enabled, any [`cadence::MetricSink`] can be used as well.

Afterwards, metrics can be recorded with the [`counter!`], [`gauge!`], or [`distribution!`] macro.

```rust
use merni::{MetricSink, StatsdRecorder, counter};

struct PrintlnSink;
impl MetricSink for PrintlnSink {
    fn emit(&self, metric: &str) {
        println!("{metric}");
    }
}

// Once when initializing:
let recorder = StatsdRecorder::new("global.prefix", PrintlnSink)
    .with_tag("global_tag", "tag_value");

merni::init(recorder).expect("global recorder already registered");

// Later on, anywhere in your code:
counter!("some.counter", 1);

// Using a dynamic counter key/name:
counter!(format_args!("some.{}", "other.counter"), 1);

// It is also possible to add tags:
counter!("with.tags", 1, "tag" => "value");
```

[`cadence`]: https://docs.rs/cadence
