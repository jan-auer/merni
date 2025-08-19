# Mérni

> Hungarian: to measure
> [wiktionary](https://en.wiktionary.org/wiki/m%C3%A9r#Hungarian)

An opinionated metrics crate with low overhead, and thread-local aggregation.

## Usage

To start recording metrics, one has to create a [`Dispatcher`], giving it an
appropriate [`Sink`] implementation.
The dispatcher can then be registered globally using [`set_global_dispatcher`].

The most efficient usage can be achieved by using the [`ThreadLocalAggregator`],
which itself acts as a [`Sink`], but has to be used together with an [`AggregationSink`].

This will aggregate metrics thread-locally, and flush them periodically to the
configured [`AggregationSink`].

Once configured, metrics can be recorded with the [`counter!`], [`gauge!`], or [`distribution!`] macro.

```rust
use merni::{Sink, Dispatcher, Metric, counter};

struct PrintlnSink;
impl Sink for PrintlnSink {
    fn emit(&self, metric: Metric) {
        let mut tags = metric.tags().peekable();
        let tags = if tags.peek().is_some() {
            let mut tags_str = String::new();
            for (i, (k,v)) in tags.enumerate() {
                tags_str.push(if i == 0 { '[' } else { ',' });
                tags_str.push_str(k);
                tags_str.push(':');
                tags_str.push_str(v);
            }
            tags_str.push(']');
            tags_str
        } else {
            String::new()
        };
        println!("{}{tags}: {}", metric.key(), metric.value().get());
    }
}

// Once when initializing:
let dispatcher = Dispatcher::new(PrintlnSink);
merni::set_global_dispatcher(dispatcher).expect("global recorder already registered");

// Later on, anywhere in your code:
counter!("some.counter": 1);

// It is also possible to add tags:
counter!("with.tags": 1, "tag" => "value");
```
