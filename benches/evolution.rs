use std::ops::ControlFlow;

use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use post_tag::{vec_deque_bools::VecDequeBools, PostSystem};

fn bench_evolve_111010<S: PostSystem>() -> impl Fn(&mut Bencher) {
    let compressed = black_box([true, true, true, false, true, false]);
    move |b| {
        b.iter(|| {
            let mut system = S::new_decompressed(&compressed);
            for _ in 0..2141 {
                system.evolve();
            }
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "VecDequeBools evolve 111010",
        bench_evolve_111010::<VecDequeBools>(),
    );
}

criterion_group!(evolution, criterion_benchmark);
criterion_main!(evolution);
