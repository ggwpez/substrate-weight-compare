use rstest::*;
use std::collections::BTreeSet as Set;

use crate::{add, mul, scope::BasicScope, term::Term, val, var};

#[rstest]
#[case(val!(123), vec![], vec![])]
#[case(mul!(var!("unbound"), val!(123)), vec![], vec!["unbound"])]
#[case(add!(var!("a"), var!("b")), vec!["b"], vec!["a"])]
fn term_free_vars_works(#[case] term: Term, #[case] bound: Vec<&str>, #[case] unbound: Vec<&str>) {
	let unbound: Set<String> = unbound.iter().cloned().map(|u| u.into()).collect();
	let mut scope = BasicScope::empty();
	for var in bound {
		scope = scope.with_var(var, val!(0)); // Just put 0 in
	}

	assert_eq!(term.free_vars(&scope), unbound);
}
