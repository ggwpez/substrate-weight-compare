#[cfg(test)]
use rstest::*;

use crate::{parse::pallet::*, scope::*, term::*, *};
use maplit::hashmap;

#[test]
fn extend_scoped_components_works() {
	// One component without range
	{
		let a = SimpleExtrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("a"),
			comp_ranges: None,
		};
		let base = SimpleScope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))]]);

		let scopes = extend_scoped_components(Some(&a), Some(&a), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))], vec![("a".into(), scalar!(100))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))], vec![("a".into(), scalar!(100))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))], vec![("a".into(), scalar!(100))]]);
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
		let a = SimpleExtrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("a"),
			comp_ranges: Some(comp_ranges),
		};
		let base = SimpleScope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))]]);

		let scopes = extend_scoped_components(Some(&a), Some(&a), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))], vec![("a".into(), scalar!(200))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))], vec![("a".into(), scalar!(200))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))], vec![("a".into(), scalar!(200))]]);

		// exact worst
		let scopes = extend_scoped_components(Some(&a), None, CompareMethod::ExactWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))], vec![("a".into(), scalar!(200))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::ExactWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))], vec![("a".into(), scalar!(200))]]);

		let scopes = extend_scoped_components(None, Some(&a), CompareMethod::ExactWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0))], vec![("a".into(), scalar!(200))]]);
	}
	// Two components without ranges
	{
		let a = SimpleExtrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("a"),
			comp_ranges: None,
		};
		let b = SimpleExtrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("b"),
			comp_ranges: None,
		};
		let base = SimpleScope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0)), ("b".into(), scalar!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(
			scopes,
			vec![
				vec![("a".into(), scalar!(0)), ("b".into(), scalar!(0))],
				vec![("a".into(), scalar!(0)), ("b".into(), scalar!(100))],
				vec![("a".into(), scalar!(100)), ("b".into(), scalar!(0))],
				vec![("a".into(), scalar!(100)), ("b".into(), scalar!(100))]
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
		let a = SimpleExtrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("a"),
			comp_ranges: Some(comp_ranges.clone()),
		};
		let b = SimpleExtrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("b"),
			comp_ranges: Some(comp_ranges),
		};
		let base = SimpleScope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0)), ("b".into(), scalar!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(
			scopes,
			vec![
				vec![("a".into(), scalar!(0)), ("b".into(), scalar!(0))],
				vec![("a".into(), scalar!(0)), ("b".into(), scalar!(100))],
				vec![("a".into(), scalar!(200)), ("b".into(), scalar!(0))],
				vec![("a".into(), scalar!(200)), ("b".into(), scalar!(100))]
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
		let a = SimpleExtrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("a"),
			comp_ranges: Some(comp_ranges.clone()),
		};
		let b = SimpleExtrinsic {
			name: "".into(),
			pallet: "".into(),
			term: var!("b"),
			comp_ranges: Some(comp_ranges.clone()),
		};
		let base = SimpleScope::empty();

		// base
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::Base, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(scopes, vec![vec![("a".into(), scalar!(0)), ("b".into(), scalar!(0))]]);
		// guess worst
		let scopes = extend_scoped_components(Some(&a), Some(&b), CompareMethod::GuessWorst, &base)
			.unwrap()
			.into_iter()
			.map(|s| s.as_vec())
			.collect::<Vec<_>>();
		assert_eq!(
			scopes,
			vec![
				vec![("a".into(), scalar!(0)), ("b".into(), scalar!(0))],
				vec![("a".into(), scalar!(0)), ("b".into(), scalar!(200))],
				vec![("a".into(), scalar!(200)), ("b".into(), scalar!(0))],
				vec![("a".into(), scalar!(200)), ("b".into(), scalar!(200))]
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
				vec![("a".into(), scalar!(0)), ("b".into(), scalar!(0))],
				vec![("a".into(), scalar!(0)), ("b".into(), scalar!(200))],
				vec![("a".into(), scalar!(200)), ("b".into(), scalar!(0))],
				vec![("a".into(), scalar!(200)), ("b".into(), scalar!(200))]
			]
		);
	}
}

#[rstest]
#[case(
	// 7.57M + 13.03M * n + 485.56K * l + 2 * READ + 2 * WRITE
	add!(add!(add!(add!(scalar!(7570000), mul!(scalar!(13030000), var!("n"))), mul!(scalar!(485560), var!("l"))), mul!(scalar!(2), var!("READ"))), mul!(scalar!(2), var!("WRITE"))),
	// 8.35M + 16.89M * n + 402.57K * l + 2 * READ + 2 * WRITE
	add!(add!(add!(add!(scalar!(8350000), mul!(scalar!(16890000), var!("n"))), mul!(scalar!(402570), var!("l"))), mul!(scalar!(2), var!("READ"))), mul!(scalar!(2), var!("WRITE"))),
	&[(CompareMethod::Asymptotic, RelativeChange::Changed, 21.706),
	  (CompareMethod::ExactWorst, RelativeChange::Changed, 24.784),
	  (CompareMethod::Base, RelativeChange::Changed, 0.302)],
)]
#[case(
	// 1K + 1K * n
	add!(scalar!(1000), mul!(scalar!(1000), var!("n"))),
	// 1K + 0.5K * n
	add!(scalar!(1000), mul!(scalar!(500), var!("n"))),
	&[(CompareMethod::Asymptotic, RelativeChange::Changed, -49.504),
	  (CompareMethod::ExactWorst, RelativeChange::Changed, 0.0),
	  (CompareMethod::Base, RelativeChange::Changed, 0.0)
	],
)]
#[case(
	// 1K + 0.5K * n
	add!(scalar!(1000), mul!(scalar!(500), var!("n"))),
	// 1K + 1K * n
	add!(scalar!(1000), mul!(scalar!(1000), var!("n"))),
	&[(CompareMethod::Asymptotic, RelativeChange::Changed, 98.039),
	  (CompareMethod::ExactWorst, RelativeChange::Changed, 98.039),
	  (CompareMethod::Base, RelativeChange::Changed, 0.0)],
)]
#[case(
	// 1K + 0.5K * n
	add!(scalar!(1000), mul!(scalar!(500), var!("n"))),
	// 1001 + 1K * n
	add!(scalar!(1001), mul!(scalar!(1000), var!("n"))),
	&[(CompareMethod::Asymptotic, RelativeChange::Changed, 98.041),
	  (CompareMethod::ExactWorst, RelativeChange::Changed, 98.041),
	  (CompareMethod::Base, RelativeChange::Changed, 0.099)],
)]
#[case(
	// 1K + 0.5K * n
	add!(scalar!(1000), mul!(scalar!(500), var!("n"))),
	// 999 + 1K * n
	add!(scalar!(999), mul!(scalar!(1000), var!("n"))),
	&[(CompareMethod::Asymptotic, RelativeChange::Changed, 98.037),
	  (CompareMethod::ExactWorst, RelativeChange::Changed, 98.037),
	  (CompareMethod::Base, RelativeChange::Changed, -0.1)],
)]
fn compare_extrinsics_works(
	#[case] old: SimpleTerm,
	#[case] new: SimpleTerm,
	#[case] expected: &[(CompareMethod, RelativeChange, f64)],
) {
	let old = SimpleExtrinsic {
		name: "".into(),
		pallet: "".into(),
		term: old,
		// n=100, l=255
		comp_ranges: Some(hashmap! {
			"n".into() => ComponentRange { min: 0, max: 100 },
			"l".into() => ComponentRange { min: 0, max: 255 },
		}),
	};
	let new = SimpleExtrinsic {
		name: "".into(),
		pallet: "".into(),
		term: new,
		// n=100, l=255
		comp_ranges: Some(hashmap! {
			"n".into() => ComponentRange { min: 0, max: 100 },
			"l".into() => ComponentRange { min: 0, max: 255 },
		}),
	};
	for expected in expected {
		let params = CompareParams {
			method: expected.0,
			unit: Dimension::Time,
			ignore_errors: false,
			git_pull: false,
			offline: true,
		};

		let change = compare_extrinsics(Some(old.clone()), Some(new.clone()), &params).unwrap();
		assert_eq!(change.change, expected.1);
		assert!(
			(change.percent - expected.2).abs() < 0.001,
			"expected: {}, got: {}",
			expected.2,
			change.percent
		);
	}
}

#[rstest]
#[case(scalar!(30), Ok(()))]
#[case(var!("READ"), Ok(()))]
#[case(mul!(var!("READ"), scalar!(1000)), Ok(()))]
#[case(mul!(var!("READ"), scalar!(1000)), Ok(()))]
#[case(mul!(var!("READ"), scalar!(1001)), Err("Call has 1001 READs"))]
#[case(mul!(var!("WRITE"), scalar!(1001)), Err("Call has 1001 WRITEs"))]
#[case(add!(var!("READ"), scalar!(1001)), Ok(()))]
#[case(add!(var!("WRITE"), scalar!(1001)), Ok(()))]
#[case(mul!(scalar!(1001), var!("WRITE")), Err("Call has 1001 WRITEs"))]
#[case(mul!(scalar!(1001), var!("READ")), Err("Call has 1001 READs"))]
#[case(mul!(var!("READ"), scalar!(2001)), Err("Call has 2001 READs"))]
#[case(mul!(var!("WRITE"), scalar!(2001)), Err("Call has 2001 WRITEs"))]
#[case(mul!(var!("SOMETHING"), scalar!(2001)), Ok(()))]
#[case(mul!(mul!(var!("READ"), scalar!(1234)), var!("READ")), Err("Call has 1234 READs"))]
#[case(mul!(mul!(var!("READ"), scalar!(1234)), mul!(var!("WRITE"), scalar!(2222))), Err("Call has 2222 WRITEs"))]
fn sanity_check_term_works(#[case] term: SimpleTerm, #[case] res: std::result::Result<(), &str>) {
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
		scope: SimpleScope::empty(),
		percent: percent(old, new),
		change: RelativeChange::Changed,
		method: CompareMethod::GuessWorst,
	}
}
