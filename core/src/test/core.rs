#[cfg(test)]
use rstest::*;

use crate::{parse::pallet::*, *};

#[test]
fn extend_scoped_components_works() {
	// One component without range
	{
		let a =
			Extrinsic { name: "".into(), pallet: "".into(), term: var!("a"), comp_ranges: None };
		let base = Scope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))]]);

		let scopes = extend_scoped_components(Some(&a), Some(&a), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))], vec![("a".into(), val!(100))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))], vec![("a".into(), val!(100))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))], vec![("a".into(), val!(100))]]);
		// exact worst
		let _err =
			extend_scoped_components(None, Some(&a), CompareMethod::ExactWorst, &base).unwrap_err();
		let _err =
			extend_scoped_components(Some(&a), None, CompareMethod::ExactWorst, &base).unwrap_err();
		let _err = extend_scoped_components(Some(&a), Some(&a), CompareMethod::ExactWorst, &base)
			.unwrap_err();
	}
	// One component with range
	{
		let mut comp_ranges = HashMap::new();
		comp_ranges.insert("a".into(), ComponentRange { min: 0, max: 200 });
		let a = Extrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("a"),
			comp_ranges: Some(comp_ranges),
		};
		let base = Scope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))]]);

		let scopes = extend_scoped_components(Some(&a), Some(&a), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))], vec![("a".into(), val!(200))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))], vec![("a".into(), val!(200))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))], vec![("a".into(), val!(200))]]);

		// exact worst
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::ExactWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))], vec![("a".into(), val!(200))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::ExactWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))], vec![("a".into(), val!(200))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::ExactWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0))], vec![("a".into(), val!(200))]]);
	}
	// Two components without ranges
	{
		let a =
			Extrinsic { name: "".into(), pallet: "".into(), term: var!("a"), comp_ranges: None };
		let b =
			Extrinsic { name: "".into(), pallet: "".into(), term: var!("b"), comp_ranges: None };
		let base = Scope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0)), ("b".into(), val!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(
			scopes,
			vec![
				vec![("a".into(), val!(0)), ("b".into(), val!(0))],
				vec![("a".into(), val!(0)), ("b".into(), val!(100))],
				vec![("a".into(), val!(100)), ("b".into(), val!(0))],
				vec![("a".into(), val!(100)), ("b".into(), val!(100))]
			]
		);
		// exact worst
		let _err = extend_scoped_components(Some(&a), Some(&b), CompareMethod::ExactWorst, &base)
			.unwrap_err();
	}
	// Two components with one range
	{
		let mut comp_ranges = HashMap::new();
		comp_ranges.insert("a".into(), ComponentRange { min: 0, max: 200 });
		let a = Extrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("a"),
			comp_ranges: Some(comp_ranges.clone()),
		};
		let b = Extrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("b"),
			comp_ranges: Some(comp_ranges),
		};
		let base = Scope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0)), ("b".into(), val!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(
			scopes,
			vec![
				vec![("a".into(), val!(0)), ("b".into(), val!(0))],
				vec![("a".into(), val!(0)), ("b".into(), val!(100))],
				vec![("a".into(), val!(200)), ("b".into(), val!(0))],
				vec![("a".into(), val!(200)), ("b".into(), val!(100))]
			]
		);
		// exact worst
		let _err = extend_scoped_components(Some(&a), Some(&b), CompareMethod::ExactWorst, &base)
			.unwrap_err();
	}
	// Two components with two ranges
	{
		let mut comp_ranges = HashMap::new();
		comp_ranges.insert("a".into(), ComponentRange { min: 0, max: 200 });
		comp_ranges.insert("b".into(), ComponentRange { min: 0, max: 200 });
		let a = Extrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("a"),
			comp_ranges: Some(comp_ranges.clone()),
		};
		let b = Extrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("b"),
			comp_ranges: Some(comp_ranges.clone()),
		};
		let base = Scope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), val!(0)), ("b".into(), val!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(
			scopes,
			vec![
				vec![("a".into(), val!(0)), ("b".into(), val!(0))],
				vec![("a".into(), val!(0)), ("b".into(), val!(200))],
				vec![("a".into(), val!(200)), ("b".into(), val!(0))],
				vec![("a".into(), val!(200)), ("b".into(), val!(200))]
			]
		);
		// exact worst
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::ExactWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(
			scopes,
			vec![
				vec![("a".into(), val!(0)), ("b".into(), val!(0))],
				vec![("a".into(), val!(0)), ("b".into(), val!(200))],
				vec![("a".into(), val!(200)), ("b".into(), val!(0))],
				vec![("a".into(), val!(200)), ("b".into(), val!(200))]
			]
		);
	}
}

#[rstest]
#[case(val!(30), Ok(()))]
#[case(var!("READ"), Ok(()))]
#[case(mul!(var!("READ"), val!(50)), Ok(()))]
#[case(mul!(var!("READ"), val!(50)), Ok(()))]
#[case(mul!(var!("READ"), val!(51)), Err("Call has 51 READs"))]
#[case(mul!(var!("WRITE"), val!(51)), Err("Call has 51 WRITEs"))]
#[case(add!(var!("READ"), val!(51)), Ok(()))]
#[case(add!(var!("WRITE"), val!(51)), Ok(()))]
#[case(mul!(val!(51), var!("WRITE")), Err("Call has 51 WRITEs"))]
#[case(mul!(val!(51), var!("READ")), Err("Call has 51 READs"))]
#[case(mul!(var!("READ"), val!(501)), Err("Call has 501 READs"))]
#[case(mul!(var!("WRITE"), val!(501)), Err("Call has 501 WRITEs"))]
#[case(mul!(var!("SOMETHING"), val!(501)), Ok(()))]
#[case(mul!(mul!(var!("READ"), val!(123)), var!("READ")), Err("Call has 123 READs"))]
#[case(mul!(mul!(var!("READ"), val!(123)), mul!(var!("WRITE"), val!(222))), Err("Call has 222 WRITEs"))]
fn sanity_check_term_works(#[case] term: Term, #[case] res: std::result::Result<(), &str>) {
	assert_eq!(sanity_check_term(&term), res.map_err(Into::into), "term: {}", term);
}

#[rstest]
#[case(10, 11, 9., true)]
#[case(10, 11, 11., false)]
#[case(673, 673, 10., false)]
#[case(100, 200, 10., true)]
#[case(100, 200, 100., true)]
#[case(100, 200, 101., false)]
fn filter_rel_threshold_works(
	#[case] old: u128,
	#[case] new: u128,
	#[case] threshold: f64,
	#[case] kept: bool,
) {
	let diffs = vec![ExtrinsicDiff {
		name: String::new(),
		file: String::new(),
		change: TermDiff::Changed(mocked_change(old, new)),
	}];
	let params = FilterParams { threshold, ..Default::default() };

	assert_eq!(
		filter_changes(diffs.clone(), &params).is_empty(),
		!kept,
		"old: {}, new: {}, threshold: {}, diffs: {:?}",
		old,
		new,
		threshold,
		diffs
	);
}

fn mocked_change(old: u128, new: u128) -> TermChange {
	TermChange {
		old: None,
		old_v: Some(old),
		new: None,
		new_v: Some(new),
		scope: Scope::empty(),
		percent: percent(old, new),
		change: RelativeChange::Changed,
		method: CompareMethod::GuessWorst,
	}
}
