//! Contains the [`Term`] which is used to express weight equations.

use std::{collections::BTreeSet as Set, fmt};
use syn::{BinOp, ExprBinary};

use crate::{fmt_weight, scope::Scope};

/// A symbolic term that can be used to express simple arithmetic.
///
/// Can only be evaluated to a concrete value within a [`crate::scope::Scope`].
///
/// ```rust
/// use swc_core::{add, mul, val, scope::MockedScope, term::Term};
///
/// // 5 * 5 + 10 == 35
/// let term = add!(mul!(val!(5), val!(5)), val!(10));
/// assert_eq!(term.eval(&MockedScope::default()), 35);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
	Value(u128),
	Var(String),

	Add(Box<Term>, Box<Term>),
	Mul(Box<Term>, Box<Term>),
}

/// Builds a [`Term::Value`] from a [`u128`].
#[macro_export]
macro_rules! val {
	($a:expr) => {
		Term::Value(($a as u128).into())
	};
}

/// Builds a [`Term::Var`] from a [`String`].
#[macro_export]
macro_rules! var {
	($a:expr) => {
		Term::Var($a.into())
	};
}

/// Builds a [`Term::Add`] from two [`Term`]s.
#[macro_export]
macro_rules! add {
	($a:expr, $b:expr) => {
		Term::Add($a.into(), $b.into())
	};
}

/// Builds a [`Term::Mul`] from two [`Term`]s.
#[macro_export]
macro_rules! mul {
	($a:expr, $b:expr) => {
		Term::Mul($a.into(), $b.into())
	};
}

impl Term {
	/// Evaluates the term within the given scope to a concrete value.
	pub fn eval(self, ctx: &crate::scope::Scope) -> Result<u128, String> {
		match self {
			Self::Value(x) => Ok(x),
			Self::Add(x, y) => Ok(x.eval(ctx)? + y.eval(ctx)?),
			Self::Mul(x, y) => Ok(x.eval(ctx)? * y.eval(ctx)?),
			Self::Var(x) =>
				if let Some(var) = ctx.get(&x) {
					var.eval(ctx)
				} else {
					Err(format!("Variable '{}' not found", x))
				},
		}
	}

	/// Returns the variables of the term that are not part of [`crate::scope::Scope`].
	///
	/// Lambda calculus calls such a variable *free*.
	/// This is the inverse of [`bound_vars`].
	pub fn free_vars(&self, scope: &Scope) -> Set<String> {
		match self {
			Self::Var(var) if scope.get(var).is_some() => Set::default(),
			Self::Var(var) => Set::from([var.into()]),

			Self::Value(_) => Set::default(),
			Self::Mul(l, r) | Self::Add(l, r) =>
				l.free_vars(scope).union(&r.free_vars(scope)).cloned().collect(),
		}
	}

	/// Returns the variables of the term that are part of [`crate::scope::Scope`].
	///
	/// Lambda calculus calls such a variable *bound*.
	/// This is the inverse of [`free_vars`].
	pub fn bound_vars(&self, scope: &Scope) -> Set<String> {
		match self {
			Self::Var(var) if scope.get(var).is_some() => Set::from([var.into()]),
			Self::Var(_var) => Set::default(),

			Self::Value(_) => Set::default(),
			Self::Mul(l, r) | Self::Add(l, r) =>
				l.bound_vars(scope).union(&r.bound_vars(scope)).cloned().collect(),
		}
	}

	pub fn fmt_equation(&self, scope: &Scope) -> String {
		let bounds = self.bound_vars(scope);
		let frees = self.free_vars(scope);

		let mut equation = Vec::<String>::new();
		for var in bounds.iter() {
			let v = scope.get(var).unwrap();
			equation.push(format!("{}={}", var, v));
		}
		for var in frees.iter() {
			equation.push(var.clone());
		}
		equation.join(", ")
	}

	fn fmt_with_bracket(&self, has_bracket: bool) -> String {
		match self {
			Self::Mul(l, r) => {
				// Omit `1 *` and `* 1`.
				if Term::Value(1) == **l {
					r.fmt_with_bracket(has_bracket)
				} else if Term::Value(1) == **r {
					l.fmt_with_bracket(has_bracket)
				} else {
					format!("{} * {}", l.fmt_with_bracket(false), r.fmt_with_bracket(false))
				}
			},
			Self::Add(l, r) => {
				// Omit `0 +` and `+ 0`.
				if Term::Value(0) == **l {
					r.fmt_with_bracket(has_bracket)
				} else if Term::Value(0) == **r {
					l.fmt_with_bracket(has_bracket)
				} else if has_bracket {
					format!("{} + {}", l.fmt_with_bracket(true), r.fmt_with_bracket(true))
				} else {
					format!("({} + {})", l.fmt_with_bracket(true), r.fmt_with_bracket(true))
				}
			},
			Self::Value(val) => fmt_weight(*val),
			Self::Var(var) => var.clone(),
		}
	}
}

/// This function is recursive to easier adopt changes.
/// It assumes that increasing variables always correlate to increasing weights.
/// We therefore only look at the corner where all variables are set to their `MAX` value.
/// The better solution would be to calculate along all edges
/// by setting all variables to `MIN` and `MAX`.
/// Calculating along the whole plane should not be needed since we assume linear equations.
/// TODO a check could be added to ensure that we are indeed dealing with linear equations.
pub fn multivariadic_eval(f: &Term, scope: &mut Scope, value: u128) -> u128 {
	let free_vars: Set<_> = f.free_vars(&scope);
	for var in free_vars {
		scope.put_var(&var, val!(value));
	}

	let _eq = f.fmt_equation(&scope);
	f.clone().eval(&scope).expect("Set all variables; qed")
}

impl fmt::Display for Term {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.fmt_with_bracket(true))
	}
}

impl TryInto<Term> for &ExprBinary {
	type Error = String;

	fn try_into(self) -> Result<Term, Self::Error> {
		let left = crate::parse::pallet::parse_expression(&self.left)?.into();
		let right = crate::parse::pallet::parse_expression(&self.right)?.into();

		let term = match self.op {
			BinOp::Mul(_) => Term::Mul(left, right),
			BinOp::Add(_) => Term::Add(left, right),
			_ => return Err("Unexpected operator".into()),
		};
		Ok(term)
	}
}
