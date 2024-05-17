use divan::Bencher;
use merni::{set_global_dispatcher, set_local_dispatcher, Dispatcher, ThreadLocalAggregator};

mod benches;

// #[global_allocator]
// static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    let sink = ThreadLocalAggregator::new();
    set_global_dispatcher(Dispatcher::new(sink)).unwrap();

    divan::main();
}

mod aggregator {
    use std::sync::Arc;

    use super::*;

    use benches::metric::*;

    #[divan::bench(items_count = 4u8)]
    fn simple_global() {
        emit_simple();
    }

    #[divan::bench]
    fn simple(bencher: Bencher) {
        let sink = Arc::new(ThreadLocalAggregator::new());
        bencher
            .counter(divan::counter::ItemsCount::new(4u8))
            .with_inputs(|| Dispatcher::new(Arc::clone(&sink)))
            .bench_values(|dispatcher| {
                let guard = set_local_dispatcher(dispatcher);
                emit_simple();
                guard.take()
            });
    }

    #[divan::bench]
    fn distribution(bencher: Bencher) {
        bencher
            .counter(divan::counter::ItemsCount::new(2u8))
            .with_inputs(|| {
                let sink = ThreadLocalAggregator::new();
                Dispatcher::new(sink)
            })
            .bench_values(|dispatcher| {
                let guard = set_local_dispatcher(dispatcher);
                emit_distribution();
                guard.take()
            });
    }
}

mod tags {
    use super::benches::tags as benches;

    #[divan::bench]
    fn _1_vec_string() {
        benches::vec_string()
    }

    #[divan::bench]
    fn _2_boxed_string() {
        benches::boxed_string()
    }

    #[divan::bench]
    fn _3_boxed_boxed() {
        benches::boxed_boxed()
    }

    #[divan::bench]
    fn _4_thread_local() {
        benches::thread_local()
    }

    #[divan::bench]
    fn _5_smallvec() {
        benches::smallvec()
    }

    #[divan::bench]
    fn _6_smolstr() {
        benches::smolstr()
    }

    #[divan::bench]
    fn _7_smallvec_smolstr() {
        benches::smallvec_smolstr()
    }

    #[divan::bench]
    fn _8_smolbuf() {
        benches::smolbuf()
    }
}
