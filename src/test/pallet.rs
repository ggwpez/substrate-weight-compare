use rstest::*;
use std::path::PathBuf;
use syn::*;

use crate::{
	add, mul,
	parse::pallet::{parse_expression, parse_file},
	reads,
	scope::MockedScope,
	term::Term,
	val, var, writes,
};

/// Parses hard-coded weight files.
#[rstest]
#[case("test_data/new/pallet_staking.rs.txt")]
#[case("test_data/old/pallet_staking.rs.txt")]
fn parses_weight_files(#[case] path: PathBuf) {
	assert!(parse_file(&path).is_ok());
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
	let _ = got.eval(&MockedScope::default());
}
