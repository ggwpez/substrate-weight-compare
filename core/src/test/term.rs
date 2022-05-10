use rstest::*;

use crate::{
	add, mul, reads,
	scope::{BasicScope, Scope},
	term::Term,
	val, var, writes,
};

#[rstest]
#[case(val!(123), vec![], vec![])]
#[case(mul!(var!("unbound"), val!(123)), vec![], vec!["unbound"])]
#[case(add!(var!("a"), var!("b")), vec!["b"], vec!["a"])]
fn term_unbound_vars_works(
	#[case] term: Term,
	#[case] bound: Vec<&str>,
	#[case] unbound: Vec<&str>,
) {
	let mut scope = BasicScope::empty();
	for var in bound {
		scope = scope.with_var(var, val!(0)); // Just put 0 in
	}

	assert_eq!(term.unbound_vars(&scope), unbound);
}
