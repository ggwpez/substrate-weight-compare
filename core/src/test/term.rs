use rstest::*;
use std::collections::BTreeSet as Set;

use crate::{add, mul, scalar, scope::SimpleScope, term::SimpleTerm, var};

#[rstest]
#[case(scalar!(123), vec![], vec![])]
#[case(mul!(var!("unbound"), scalar!(123)), vec![], vec!["unbound"])]
#[case(add!(var!("a"), var!("b")), vec!["b"], vec!["a"])]
fn term_free_vars_works(
	#[case] term: SimpleTerm,
	#[case] bound: Vec<&str>,
	#[case] unbound: Vec<&str>,
) {
	let unbound: Set<String> = unbound.iter().cloned().map(|u| u.into()).collect();
	let mut scope = SimpleScope::empty();
	for var in bound {
		scope = scope.with_var(var, scalar!(0)); // Just put 0 in
	}

	assert_eq!(term.free_vars(&scope), unbound);
}

const F: &str = "F";

#[rstest]
#[case(scalar!(123), None)]
#[case(var!(F), None)]
#[case(add!(var!(F), scalar!(4)), None)]
#[case(mul!(scalar!(4), var!(F)), Some(4))]
#[case(mul!(var!(F), scalar!(4)), Some(4))]
#[case(add!(mul!(scalar!(5), var!(F)), mul!(var!(F), scalar!(5))), Some(5))]
fn term_find_largest_factor_works(#[case] term: SimpleTerm, #[case] largest: Option<u128>) {
	assert_eq!(term.find_largest_factor(F), largest, "term: {}", term);
}

#[rstest]
#[case(scalar!(123), "123")]
#[case(var!("x"), "x")]
#[case(add!(var!("x"), scalar!(123)), "x + 123")]
#[case(add!(var!("x"), scalar!(0)), "x")]
#[case(mul!(var!("x"), scalar!(123)), "x * 123")]
#[case(mul!(var!("x"), scalar!(1)), "x")]
#[case(add!(add!(scalar!(0), scalar!(0)), scalar!(0)), "0")]
#[case(add!(add!(scalar!(0), scalar!(0)), add!(scalar!(0), scalar!(0))), "0")]
// TODO test rounding
#[case(add!(scalar!(12_680_000), add!(
	mul!(scalar!(4_410_000), var!("c")),
	add!(mul!(scalar!(1), var!("READ")), mul!(var!("WRITE"), scalar!(1)))
)),
"12.68M + 4.41M * c + READ + WRITE")]
#[case(mul!(var!("m"), var!("READ")), "m * READ")]
#[case(mul!(mul!(scalar!(1), var!("m")), var!("READ")), "m * READ")]
#[case(add!(add!(add!(add!(scalar!(9909), mul!(scalar!(0), var!("l"))), mul!(scalar!(0), var!("s"))), mul!(scalar!(3), var!("READ"))), mul!(scalar!(3), var!("WRITE"))), "9.91K + 3 * READ + 3 * WRITE")]
fn term_fmt_with_brackets_works(#[case] term: SimpleTerm, #[case] expected: &str) {
	assert_eq!(format!("{}", term), expected);
}

/*#[case(scalar!(123), scalar!(123))]
fn test_substitute_works(#[case] term: SimpleTern, #[case] expected: SimpleTerm) {
	term
}*/
