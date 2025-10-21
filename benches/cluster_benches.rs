use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graph_test::{clustering, prepare_routers};

fn bench_optimize(c: &mut Criterion) {
    for seed in 100..110 {
        // ベンチマーク名をユニークにする
        let (d, time_exec, routers_template) = prepare_routers(seed, 500.0);
        let bench_name = format!("benchmark_cluster_seed_{}_with_450_devices", seed);
        let k = 3;

        c.bench_function(&bench_name, |b| {
            b.iter(|| {
                let mut routers = routers_template.clone();
                clustering(
                    k,
                    black_box(&d),
                    black_box(&time_exec),
                    black_box(&mut routers),
                );
            })
        });
    }
}

criterion_group!(benches, bench_optimize);
criterion_main!(benches);
