//! Measures the throughput of parsing an average extrinsic.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::path::Path;

use swc_core::{add, mul, scope::Scope, term::*, val, var};

fn bench_term_add(c: &mut Criterion) {
	let mut group = c.benchmark_group("Term");
	let heigh = 14_u64;
	let size = 1 << heigh;

	let mut term = add!(var!("x"), var!("y"));
	for _ in 0..heigh {
		term = add!(term.clone(), term);
	}
	let scope = Scope::empty().with_var("x", val!(245234)).with_var("y", val!(245231));

	group.throughput(Throughput::Elements(size as u64));
	group.bench_function("Add", |b| b.iter(|| term.eval(&scope).expect("must work")));
}

criterion_group! {
	name = benches;
	config = Criterion::default();
	targets = bench_term_add
}
criterion_main!(benches);
