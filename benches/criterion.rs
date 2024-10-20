use codspeed_criterion_compat::{criterion_group, BatchSize, Criterion};
use merni::{set_global_dispatcher, set_local_dispatcher, Dispatcher, ThreadLocalAggregator};

mod benches;

fn main() {
    let sink = ThreadLocalAggregator::new();
    set_global_dispatcher(Dispatcher::new(sink)).unwrap();

    #[cfg(not(codspeed))]
    {
        tags();
        aggregator();

        Criterion::default().configure_from_args().final_summary();
    }
    #[cfg(codspeed)]
    {
        let mut criterion = Criterion::new_instrumented();
        tags(&mut criterion);
        aggregator(&mut criterion);
    }
}

criterion_group!(
    aggregator,
    aggregator::simple_global,
    aggregator::simple,
    aggregator::distribution
);
criterion_group!(
    tags,
    tags::_1_vec_string,
    tags::_2_boxed_string,
    tags::_3_boxed_boxed,
    tags::_4_thread_local,
    tags::_5_smallvec,
    tags::_6_smolstr,
    tags::_7_smallvec_smolstr,
    tags::_8_smolbuf,
);

mod aggregator {
    use std::sync::Arc;

    use super::*;

    use benches::metric::*;

    pub fn simple_global(c: &mut Criterion) {
        c.bench_function("simple global emit", |b| b.iter(emit_simple));
    }

    pub fn simple(c: &mut Criterion) {
        let sink = Arc::new(ThreadLocalAggregator::new());
        c.bench_function("simple local emit", |b| {
            b.iter_batched(
                || Dispatcher::new(Arc::clone(&sink)),
                |dispatcher| {
                    let guard = set_local_dispatcher(dispatcher);
                    emit_simple();
                    guard.take()
                },
                BatchSize::SmallInput,
            )
        });
    }

    pub fn distribution(c: &mut Criterion) {
        c.bench_function("local distribution", |b| {
            b.iter_batched(
                || {
                    let sink = ThreadLocalAggregator::new();
                    Dispatcher::new(sink)
                },
                |dispatcher| {
                    let guard = set_local_dispatcher(dispatcher);
                    emit_distribution();
                    guard.take()
                },
                BatchSize::SmallInput,
            )
        });
    }
}

mod tags {
    use super::benches::tags as benches;
    use super::*;

    pub fn _1_vec_string(c: &mut Criterion) {
        c.bench_function("#1 Vec<String>", |b| b.iter(benches::vec_string));
    }

    pub fn _2_boxed_string(c: &mut Criterion) {
        c.bench_function("#2 Box<[String]>", |b| b.iter(benches::boxed_string));
    }

    pub fn _3_boxed_boxed(c: &mut Criterion) {
        c.bench_function("#3 Box<[Box<str>]>", |b| b.iter(benches::boxed_boxed));
    }

    pub fn _4_thread_local(c: &mut Criterion) {
        c.bench_function("#4 thread_local!", |b| b.iter(benches::thread_local));
    }

    pub fn _5_smallvec(c: &mut Criterion) {
        c.bench_function("#5 SmallVec", |b| b.iter(benches::smallvec));
    }

    pub fn _6_smolstr(c: &mut Criterion) {
        c.bench_function("#6 SmolStr", |b| b.iter(benches::smolstr));
    }

    pub fn _7_smallvec_smolstr(c: &mut Criterion) {
        c.bench_function("#7 SmallVec + SmolStr", |b| {
            b.iter(benches::smallvec_smolstr)
        });
    }

    pub fn _8_smolbuf(c: &mut Criterion) {
        c.bench_function("#8 SmallVec + SmolBuf", |b| b.iter(benches::smolbuf));
    }
}
