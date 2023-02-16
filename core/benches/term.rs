//! Measures the throughput of parsing an average extrinsic.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use swc_core::{add, scalar, scope::SimpleScope, var};

fn bench_term_add(c: &mut Criterion) {
	let mut group = c.benchmark_group("Term");
	let heigh = 14_u64;
	let size = 1 << heigh;

	let mut term = add!(var!("x"), var!("y"));
	for _ in 0..heigh {
		term = add!(term.clone(), term);
	}
	let scope = SimpleScope::empty()
		.with_var("x", scalar!(245234))
		.with_var("y", scalar!(245231));

	group.throughput(Throughput::Elements(size as u64));
	group.bench_function("Add", |b| b.iter(|| term.eval(&scope).expect("must work")));
}

criterion_group! {
	name = benches;
	config = Criterion::default();
	targets = bench_term_add
}
criterion_main!(benches);
