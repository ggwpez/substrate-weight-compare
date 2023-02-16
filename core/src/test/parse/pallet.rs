use rstest::*;
use std::{collections::HashMap, path::PathBuf};
use syn::*;

use crate::{
	add, cadd, cmul, creads, cval, cvar, cwrites, mul,
	parse::pallet::{
		parse_content, parse_expression, parse_file, parse_scalar_expression, ChromaticExtrinsic,
		ComponentRange, GenericExtrinsic,
	},
	reads, scalar,
	scope::{GenericScope, *},
	term::{ChromaticTerm, GenericTerm, SimpleTerm},
	traits::Weight,
	val, var, writes,
};

/// Parses hard-coded weight files.
#[rstest]
#[case("../test_data/new/pallet_staking.rs.txt")]
#[case("../test_data/old/pallet_staking.rs.txt")]
#[case("../test_data/new/staking_chromatic.rs.txt")]
#[case("/home/vados/work/swc/test_data/new/staking_chromatic.rs.txt")]
fn parses_weight_files(#[case] path: PathBuf) {
	if let Err(err) = parse_file(&path) {
		panic!("Failed to parse file: {:?} with error: {:?}", path, err);
	}
}

#[rstest]
#[case(
	"impl WeightInfo for () { \
	fn ext() -> Weight { \
    	((5 as Weight)) \
	} \
}"
)]
#[case(
	"impl<T: frame_system::Config> my_pallet::WeightInfo for WeightInfo<T> { \
	fn ext() -> Weight { \
    	5 as Weight \
	} \
}"
)]
fn parse_function_v1_works(#[case] input: String) {
	let got = parse_content("".into(), input).unwrap();

	let want = vec![ChromaticExtrinsic {
		name: "ext".into(),
		pallet: "".into(),
		term: GenericTerm::Value((5, 0).into()),
		comp_ranges: None,
	}];
	assert_eq!(want, got);
}

#[rstest]
#[case(
	"impl WeightInfo for () {
	fn ext() -> Weight {
    	Weight::from_ref_time(5)
	}
}",
	5,
	0
)]
#[case(
	"impl WeightInfo for () {
	fn ext() -> Weight {
    	Weight::from_proof_size(5)
	}
}",
	0,
	5
)]
#[case(
	"impl<T: frame_system::Config> my_pallet::WeightInfo for WeightInfo<T> {
	fn ext() -> Weight {
    	Weight::from_parts(5, 0)
	}
}",
	5,
	0
)]
#[case(
	"impl<T: frame_system::Config> my_pallet::WeightInfo for WeightInfo<T> {
		/// Storage: Staking MinCommission (r:1 w:0)
		/// Proof: Staking MinCommission (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
		/// Storage: Staking Validators (r:1 w:1)
		/// Proof: Staking Validators (max_values: None, max_size: Some(45), added: 2520, mode: MaxEncodedLen)
		fn ext() -> Weight {
			//  Measured:  `694`
			//  Estimated: `3019`
			// Minimum execution time: 14_703 nanoseconds.
			Weight::from_parts(15, 30)
		}
}",
	15, 30
)]
fn parse_chromatic_function_works(#[case] input: String, #[case] t: u64, #[case] p: u64) {
	let got = parse_content("".into(), input).unwrap();

	let want = vec![ChromaticExtrinsic {
		name: "ext".into(),
		pallet: "".into(),
		term: GenericTerm::Value((t as u128, p as u128).into()),
		comp_ranges: None,
	}];
	assert_eq!(want, got);
}

// NOTE: Try not to put // into a multiline comment, it will break!
// Rather use the r# syntax.

#[rstest]
#[case(
	r#"impl<T: frame_system::Config> my_pallet::WeightInfo for WeightInfo<T> {
		/// The range of component `c` is `[1_337, 2000]`.
		/// The range of component `d` is `[42, 999999]`.
		fn ext(c: u32, d: u32, ) -> Weight {
			(5 as Weight)
		}
	}"#
)]
fn parse_component_range_works(#[case] input: String) {
	let got = parse_content("".into(), input).unwrap();

	let ranges = HashMap::from([
		("c".into(), ComponentRange { min: 1_337, max: 2000 }),
		("d".into(), ComponentRange { min: 42, max: 999_999 }),
	]);
	let want = vec![ChromaticExtrinsic {
		name: "ext".into(),
		pallet: "".into(),
		term: GenericTerm::Value((5, 0).into()),
		comp_ranges: Some(ranges),
	}];
	assert_eq!(want, got);
}

#[rstest]
// Basic arithmetic.
#[case("(123 as Weight)",
	scalar!(123))]
#[case("(123 as Weight)\
	.saturating_add(6 as Weight)",
	add!(scalar!(123), scalar!(6)))]
#[case("(123 as Weight)\
	.saturating_mul(5 as Weight)\
	.saturating_add(6 as Weight)",
	 add!(mul!(scalar!(123), scalar!(5)), scalar!(6)))]
#[case("(123 as Weight)\
	.saturating_mul(5 as Weight)\
	.saturating_add(e as Weight)\
	.saturating_mul(7 as Weight)",
	mul!(add!(mul!(scalar!(123), scalar!(5)), var!("e")), scalar!(7)))]
// Arithmetic with vars.
#[case("(123 as Weight)
	.saturating_mul(WEIGHT_PER_NANOS)",
	mul!(scalar!(123), var!("WEIGHT_PER_NANOS")))]
#[case("(123 as Weight)
	.saturating_add(a as Weight)
	.saturating_mul(m)",
	mul!(add!(scalar!(123), var!("a")), var!("m")))]
// DB reads+writes.
#[case("T::DbWeight::get().reads(2 as Weight)",
	reads!(scalar!(2)))]
#[case("T::DbWeight::get().writes(2 as Weight)",
	writes!(scalar!(2)))]
#[case("T::DbWeight::get().reads(2 as Weight).saturating_add(3 as Weight)",
	add!(reads!(scalar!(2)), scalar!(3)))]
#[case("T::DbWeight::get().writes(2 as Weight).saturating_mul(3 as Weight)",
	mul!(writes!(scalar!(2)), scalar!(3)))]
#[case("T::DbWeight::get().writes(2 as Weight).saturating_add(3 as Weight)",
	add!(writes!(scalar!(2)), scalar!(3)))]
// All together.
#[case("(123 as Weight)
	// Random comment
	.saturating_add((7 as Weight).saturating_mul(s as Weight))
	.saturating_add(T::DbWeight::get().reads(12 as Weight))
	.saturating_add(T::DbWeight::get().writes(12))
	.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))",
	add!(add!(add!(add!(scalar!(123), mul!(scalar!(7), var!("s"))), reads!(scalar!(12))), writes!(scalar!(12))), writes!(mul!(scalar!(1), var!("s")))))]
fn parse_expression_works(#[case] input: &str, #[case] want: SimpleTerm) {
	let expr: Expr = syn::parse_str(input).unwrap();
	let got = parse_scalar_expression(&expr).unwrap();
	assert_eq!(want, got);

	// Eval does not panic
	let _ = got.eval(&GenericScope::empty());
}

// V1.5 syntax
#[rstest]
#[case("Weight::zero()", val!(0))]
#[case("Weight::zero().saturating_mul(Weight::from_ref_time(123))", mul!(val!(0), scalar!(123)))]
#[case("Weight::from_ref_time(123 as u64)", scalar!(123))]
#[case("Weight::from_ref_time(123 as u64)
	// Standard Error: 1_000
	.saturating_add(Weight::from_ref_time(7 as u64).saturating_mul(s as u64))
	.saturating_add(T::DbWeight::get().reads(12 as u64))
	.saturating_add(T::DbWeight::get().writes(12 as u64))
	.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(s as u64)))",
	add!(add!(add!(add!(scalar!(123), mul!(scalar!(7), var!("s"))), reads!(scalar!(12))), writes!(scalar!(12))), writes!(mul!(scalar!(1), var!("s")))))]
fn parse_expression_works_v15(#[case] input: &str, #[case] want: SimpleTerm) {
	let expr: Expr = syn::parse_str(input).unwrap();
	let got = parse_scalar_expression(&expr).unwrap();
	assert_eq!(want, got);

	// Eval does not panic
	let _ = got.eval(&SimpleScope::empty());
}

#[rstest]
#[case("Weight::from_ref_time(123)", GenericTerm::Value((123, 0).into()))]
#[case("Weight::from_proof_size(123)", GenericTerm::Value((0, 123).into()))]
#[case("Weight::from_parts(123, 321)", GenericTerm::Value((123, 321).into()))]
#[case("Weight::from_parts(48_314_000, 2603)
	.saturating_add(RocksDbWeight::get().reads(1_u64))", GenericTerm::Add(
		Box::new(GenericTerm::Value((48_314_000, 2603).into())),
		Box::new(creads!(GenericTerm::Scalar(1))),
	))]
#[case("Weight::from_parts(33_236_000, 3054)
	.saturating_add(T::DbWeight::get().reads(2_u64))
	.saturating_add(T::DbWeight::get().writes(5_u64))",
	GenericTerm::Add(
		Box::new(GenericTerm::Add(
			Box::new(GenericTerm::Value((33_236_000, 3054).into())),
			Box::new(creads!(GenericTerm::Scalar(2))),
		)),
		Box::new(cwrites!(GenericTerm::Scalar(5))),
	))
]
#[case("Weight::from_parts(890_989_741, 69146)
// Standard Error: 58_282
.saturating_add(Weight::from_ref_time(4_920_413).saturating_mul(s.into()))
.saturating_add(RocksDbWeight::get().reads(1_u64))
.saturating_add(RocksDbWeight::get().writes(1_u64))",
	GenericTerm::Add(
		Box::new(GenericTerm::Add(
			Box::new(GenericTerm::Add(
				Box::new(GenericTerm::Value((890_989_741, 69146).into())),
				Box::new(GenericTerm::Mul(
					Box::new(GenericTerm::Value((4_920_413, 0).into())),
					Box::new(GenericTerm::Var("s".into())),
				)),
			)),
			Box::new(creads!(GenericTerm::Scalar(1))),
		)),
		Box::new(cwrites!(GenericTerm::Scalar(1))),
	))]
#[case("Weight::from_parts(10, 20)
	.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))",
	GenericTerm::Add(
		Box::new(GenericTerm::Value((10, 20).into())),
		Box::new(cwrites!(GenericTerm::Mul(
			Box::new(GenericTerm::Value(Weight{time: 1, proof: 0})),
			Box::new(GenericTerm::Var("s".into())),
		))),
))]
fn chromatic_syntax(#[case] input: &str, #[case] want: ChromaticTerm) {
	let expr: Expr = syn::parse_str(input).unwrap();
	let got = parse_expression(&expr).unwrap();
	assert_eq!(want, got);

	// Eval does not panic
	let _ = got.eval(&GenericScope::empty());
}
