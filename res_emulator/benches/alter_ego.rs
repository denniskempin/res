use std::path::Path;

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BatchSize;
use criterion::Criterion;
use res_emulator::System;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("frame_time", |b| {
        b.iter_batched_ref(
            || {
                let system = System::with_ines(Path::new("tests/e2e/alter_ego.nes")).unwrap();
                system
            },
            |system: &mut System| {
                system.execute_one_frame().unwrap();
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
