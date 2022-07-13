#[cfg(test)]
use crate::{parse::pallet::*, *};

#[test]
fn test_extend_scoped_components() {
	let a = Extrinsic { name: "".into(), pallet: "".into(), term: var!("a"), comp_ranges: None };

	let mut scope = Scope::empty();
	extend_scoped_components(Some(&a), None, CompareMethod::Base, &mut scope).unwrap();
	assert_eq!(scope.get("a"), Some(val!(0)));
	let mut scope = Scope::empty();

	extend_scoped_components(None, Some(&a), CompareMethod::Base, &mut scope).unwrap();
	assert_eq!(scope.get("a"), Some(val!(0)));

	let mut scope = Scope::empty();
	extend_scoped_components(Some(&a), Some(&a), CompareMethod::Base, &mut scope).unwrap();
	assert_eq!(scope.get("a"), Some(val!(0)));

	let mut scope = Scope::empty();
	extend_scoped_components(Some(&a), None, CompareMethod::GuessWorst, &mut scope).unwrap();
	assert_eq!(scope.get("a"), Some(val!(100)));

	let mut scope = Scope::empty();
	extend_scoped_components(None, Some(&a), CompareMethod::GuessWorst, &mut scope).unwrap();
	assert_eq!(scope.get("a"), Some(val!(100)));

	let mut scope = Scope::empty();
	extend_scoped_components(Some(&a), Some(&a), CompareMethod::GuessWorst, &mut scope).unwrap();
	assert_eq!(scope.get("a"), Some(val!(100)));

	let mut scope = Scope::empty();
	extend_scoped_components(Some(&a), None, CompareMethod::ExactWorst, &mut scope).unwrap_err();
	extend_scoped_components(None, Some(&a), CompareMethod::ExactWorst, &mut scope).unwrap_err();
	extend_scoped_components(Some(&a), Some(&a), CompareMethod::ExactWorst, &mut scope)
		.unwrap_err();
}
