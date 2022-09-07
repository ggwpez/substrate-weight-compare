use rstest::*;
use std::{collections::HashMap, path::PathBuf};
use syn::*;

use crate::{
	add, mul,
	parse::pallet::{parse_content, parse_expression, parse_file, ComponentRange, Extrinsic},
	reads,
	scope::Scope,
	term::Term,
	val, var, writes,
};

/// Parses hard-coded weight files.
#[rstest]
#[case("../test_data/new/pallet_staking.rs.txt")]
#[case("../test_data/old/pallet_staking.rs.txt")]
fn parses_weight_files(#[case] path: PathBuf) {
	assert!(parse_file(&path).is_ok());
}

#[rstest]
#[case(
	"impl WeightInfo for () { \
	fn ext() -> Weight { \
    	5 \
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
fn parse_function_works(#[case] input: String) {
	let got = parse_content("".into(), input).unwrap();

	let want =
		vec![Extrinsic { name: "ext".into(), pallet: "".into(), term: val!(5), comp_ranges: None }];
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
	let want = vec![Extrinsic {
		name: "ext".into(),
		pallet: "".into(),
		term: val!(5),
		comp_ranges: Some(ranges),
	}];
	assert_eq!(want, got);
}

#[rstest]
// Basic arithmetic.
#[case("(123 as Weight)",
	val!(123))]
#[case("(123 as Weight)\
	.saturating_add(6 as Weight)",
	add!(val!(123), val!(6)))]
#[case("(123 as Weight)\
	.saturating_mul(5 as Weight)\
	.saturating_add(6 as Weight)",
	 add!(mul!(val!(123), val!(5)), val!(6)))]
#[case("(123 as Weight)\
	.saturating_mul(5 as Weight)\
	.saturating_add(e as Weight)\
	.saturating_mul(7 as Weight)",
	mul!(add!(mul!(val!(123), val!(5)), var!("e")), val!(7)))]
// Arithmetic with vars.
#[case("(123 as Weight)
	.saturating_mul(WEIGHT_PER_NANOS)",
	mul!(val!(123), var!("WEIGHT_PER_NANOS")))]
#[case("(123 as Weight)
	.saturating_add(a as Weight)
	.saturating_mul(m)",
	mul!(add!(val!(123), var!("a")), var!("m")))]
// DB reads+writes.
#[case("T::DbWeight::get().reads(2 as Weight)",
	reads!(val!(2)))]
#[case("T::DbWeight::get().writes(2 as Weight)",
	writes!(val!(2)))]
#[case("T::DbWeight::get().reads(2 as Weight).saturating_add(3 as Weight)",
	add!(reads!(val!(2)), val!(3)))]
#[case("T::DbWeight::get().writes(2 as Weight).saturating_mul(3 as Weight)",
	mul!(writes!(val!(2)), val!(3)))]
#[case("T::DbWeight::get().writes(2 as Weight).saturating_add(3 as Weight)",
	add!(writes!(val!(2)), val!(3)))]
// All together.
#[case("(123 as Weight)
	// Random comment
	.saturating_add((7 as Weight).saturating_mul(s as Weight))
	.saturating_add(T::DbWeight::get().reads(12 as Weight))
	.saturating_add(T::DbWeight::get().writes(12))
	.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))",
	add!(add!(add!(add!(val!(123), mul!(val!(7), var!("s"))), reads!(val!(12))), writes!(val!(12))), writes!(mul!(val!(1), var!("s")))))]
fn parse_expression_works(#[case] input: &str, #[case] want: Term) {
	let expr: Expr = syn::parse_str(input).unwrap();
	let got = parse_expression(&expr).unwrap();
	assert_eq!(want, got);

	// Eval does not panic
	let _ = got.eval(&Scope::empty());
}

// V1.5 syntax
#[rstest]
// Basic arithmetic.
#[case("Weight::from_ref_time(123 as u64)", val!(123))]
// All together.
#[case("Weight::from_ref_time(123 as u64)
	// Standard Error: 1_000
	.saturating_add(Weight::from_ref_time(7 as u64).saturating_mul(s as u64))
	.saturating_add(T::DbWeight::get().reads(12 as u64))
	.saturating_add(T::DbWeight::get().writes(12 as u64))
	.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(s as u64)))",
	add!(add!(add!(add!(val!(123), mul!(val!(7), var!("s"))), reads!(val!(12))), writes!(val!(12))), writes!(mul!(val!(1), var!("s")))))]
fn v1_5_parse_expression_works(#[case] input: &str, #[case] want: Term) {
	let expr: Expr = syn::parse_str(input).unwrap();
	let got = parse_expression(&expr).unwrap();
	assert_eq!(want, got);

	// Eval does not panic
	let _ = got.eval(&Scope::empty());
}
