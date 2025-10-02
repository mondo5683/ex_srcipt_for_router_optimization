use criterion::{criterion_group, criterion_main, Criterion, black_box};
use graph_test::{prepare_routers, clustering};

fn bench_optimize(c: &mut Criterion) {
    // 100〜109 のシードでループ
    /*
    for seed in 100..110 {
*/
        // ベンチマーク名をユニークにする
        let (d, time_exec, routers_template) = prepare_routers(100, 500.0);
        let k = 4;
        let bench_name = format!("benchmark_cluster");

        c.bench_function(&bench_name, |b| {
            b.iter(|| {
                let mut routers = routers_template.clone();
                clustering(k, black_box(&d), black_box(&time_exec),black_box(&mut routers));
            })
        });
    
}

criterion_group!(benches, bench_optimize);
criterion_main!(benches);

