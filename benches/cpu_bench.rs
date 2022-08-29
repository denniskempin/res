use std::path::Path;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use ners::nes::System;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("nestest", |b| {
        b.iter_batched_ref(
            || {
                let mut system = System::with_ines(Path::new("tests/cpu/nestest.nes")).unwrap();
                system.cpu.program_counter = 0xC000;
                system
            },
            |system: &mut System| {
                let mut counter = 0;
                while system.cpu.execute_one().unwrap() {
                    counter += 1;
                }
                assert_eq!(counter, 8992);
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
