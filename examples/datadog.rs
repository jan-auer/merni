use merni::{counter, distribution};

#[tokio::main]
async fn main() {
    let flusher = merni::init_datadog(None).unwrap();

    for _ in 0..10 {
        counter!("merni.test.counter": 1);
        distribution!("merni.test.distribution": 1);
    }

    flusher.flush(None).await.unwrap();
}
