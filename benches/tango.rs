use merni::{Dispatcher, set_global_dispatcher};
use tango_bench::{IntoBenchmarks, benchmark_fn, tango_benchmarks, tango_main};

mod benches;
use benches::*;

fn factorial_benchmarks() -> impl IntoBenchmarks {
    set_global_dispatcher(Dispatcher::new(noop_aggregator())).unwrap();
    [
        benchmark_fn("simple_global", |b| b.iter(emit_simple)),
        benchmark_fn("distribution_global", |b| b.iter(emit_distribution)),
    ]
}

tango_benchmarks!(factorial_benchmarks());
tango_main!();
