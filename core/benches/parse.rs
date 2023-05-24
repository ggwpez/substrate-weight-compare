//! Measures the throughput of parsing an average extrinsic.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::path::Path;

use subweight-core::parse::{pallet::parse_file as parse_pallet, storage::parse_file as parse_storage};

fn bench_parse_pallet(c: &mut Criterion) {
	let path = Path::new("../test_data/new/pallet_staking.rs.txt");
	let num_ext = parse_pallet(path).expect("Must work").len();
	let mut group = c.benchmark_group("Parse");

	group.sample_size(100);
	group.throughput(Throughput::Elements(num_ext as u64));
	group.bench_function("Pallet.Extrinsic", |b| {
		b.iter(|| parse_pallet(black_box(path)).expect("Must work"))
	});
}

fn bench_parse_storage(c: &mut Criterion) {
	let path = Path::new("../test_data/new/rocksdb_weights.rs.txt");
	let mut group = c.benchmark_group("Parse");

	group.sample_size(100);
	group.throughput(Throughput::Elements(1));
	group.bench_function("Storage", |b| {
		b.iter(|| parse_storage(black_box(path)).expect("Must work"))
	});
}

criterion_group! {
	name = benches;
	config = Criterion::default();
	targets = bench_parse_pallet, bench_parse_storage
}
criterion_main!(benches);
