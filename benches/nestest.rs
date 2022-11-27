use std::path::Path;

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BatchSize;
use criterion::Criterion;
use res::nes::System;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("nestest duration", |b| {
        b.iter_batched_ref(
            || {
                let mut system = System::with_ines(Path::new("tests/cpu/nestest.nes")).unwrap();
                system.cpu.program_counter = 0xC000;
                system
            },
            |system: &mut System| {
                system
                    .execute_until(|cpu| cpu.program_counter == 0xC6A9)
                    .unwrap();
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
