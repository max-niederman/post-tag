use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use post_tag::{bitstring::BitString, vec_deque_bools::VecDequeBools, PostSystem};

fn bench_evolve_111010<S: PostSystem>() -> impl Fn(&mut Bencher) {
    let compressed = black_box([true, true, true, false, true, false]);
    move |b| {
        b.iter(|| {
            let mut system = S::new_decompressed(&compressed);
            system.evolve_multi(2141);
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "VecDequeBools evolve 111010",
        bench_evolve_111010::<VecDequeBools>(),
    );

    c.bench_function(
        "BitString evolve 111010",
        bench_evolve_111010::<BitString>(),
    );
}

criterion_group!(evolution, criterion_benchmark);
criterion_main!(evolution);
