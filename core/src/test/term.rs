use rstest::*;
use std::collections::BTreeSet as Set;

use crate::{add, mul, scope::Scope, term::Term, val, var};

#[rstest]
#[case(val!(123), vec![], vec![])]
#[case(mul!(var!("unbound"), val!(123)), vec![], vec!["unbound"])]
#[case(add!(var!("a"), var!("b")), vec!["b"], vec!["a"])]
fn term_free_vars_works(#[case] term: Term, #[case] bound: Vec<&str>, #[case] unbound: Vec<&str>) {
	let unbound: Set<String> = unbound.iter().cloned().map(|u| u.into()).collect();
	let mut scope = Scope::empty();
	for var in bound {
		scope = scope.with_var(var, val!(0)); // Just put 0 in
	}

	assert_eq!(term.free_vars(&scope), unbound);
}

const F: &str = "F";

#[rstest]
#[case(val!(123), None)]
#[case(var!(F), None)]
#[case(add!(var!(F), val!(4)), None)]
#[case(mul!(val!(4), var!(F)), Some(4))]
#[case(mul!(var!(F), val!(4)), Some(4))]
#[case(add!(mul!(val!(5), var!(F)), mul!(var!(F), val!(5))), Some(5))]
fn term_find_largest_factor_works(#[case] term: Term, #[case] largest: Option<u128>) {
	assert_eq!(term.find_largest_factor(F), largest, "term: {}", term);
}
