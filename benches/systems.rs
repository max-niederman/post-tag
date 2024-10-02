use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use post_tag::{bitstring::BitString, vec_deque_bools::VecDequeBools, PostSystem};

fn bench_evolve_5854<S: PostSystem>() -> impl Fn(&mut Bencher) {
    let compressed = black_box([
        true, false, true, true, false, true, true, false, true, true, true, true, false,
    ]);
    move |b| {
        b.iter(|| {
            let mut system = S::new_decompressed(&compressed);
            system.evolve_multi(341_992);
        });
    }
}

fn bench_floyd_5854<S: PostSystem>() -> impl Fn(&mut Bencher) {
    let compressed = black_box([
        true, false, true, true, false, true, true, false, true, true, true, true, false,
    ]);
    move |b| {
        b.iter(|| {
            let mut tortoise = S::new_decompressed(&compressed);
            let mut hare = tortoise.clone();

            loop {
                tortoise.evolve_multi(S::PREFERRED_TIMESTEP as _);
                hare.evolve_multi(S::PREFERRED_TIMESTEP as usize + 1);

                if tortoise == hare {
                    break;
                }
            }
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "VecDequeBools evolve 5854",
        bench_evolve_5854::<VecDequeBools>(),
    );

    c.bench_function("BitString evolve 5854", bench_evolve_5854::<BitString>());

    c.bench_function(
        "VecDequeBools floyd 5854",
        bench_floyd_5854::<VecDequeBools>(),
    );

    c.bench_function("BitString floyd 5854", bench_floyd_5854::<BitString>());
}

criterion_group!(evolution, criterion_benchmark);
criterion_main!(evolution);
