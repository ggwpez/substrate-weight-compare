#[cfg(test)]
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
