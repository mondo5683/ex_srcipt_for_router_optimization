use criterion::{criterion_group, criterion_main, Criterion, black_box};
use graph_test::{prepare_routers, optimize_obvalue_in_cluster};

fn bench_optimize(c: &mut Criterion) {
    // 準備は一度だけ
    let routers_template = prepare_routers(100, 500.0);

    c.bench_function("optimize_obvalue_in_cluster", |b| {
        b.iter(|| {
            // 毎回 fresh な状態が必要なので clone
            let mut routers = routers_template.clone();
            optimize_obvalue_in_cluster(black_box(&mut routers));
        })
    });
}

criterion_group!(benches, bench_optimize);
criterion_main!(benches);

