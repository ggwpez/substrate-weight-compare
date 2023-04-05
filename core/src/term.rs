//! Contains the [`Term`] which is used to express weight equations.

use crate::traits::{One, Zero};
use lazy_static::__Deref;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet as Set, fmt};
use syn::{BinOp, ExprBinary};

use crate::{scope::Scope, traits::*};

/// A symbolic term that can be used to express simple arithmetic.
///
/// Can only be evaluated to a concrete value within a [`crate::scope::Scope`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub enum Term<T> {
	Value(T),
	Scalar(u128),
	Var(VarValue),

	Add(Box<Self>, Box<Self>),
	Mul(Box<Self>, Box<Self>),
}

pub type SimpleTerm = Term<u128>;
pub type ChromaticTerm = Term<Weight>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, Eq)]
/// A `VarValue` is an opaque string.
pub struct VarValue(pub String);

impl From<VarValue> for String {
	fn from(v: VarValue) -> String {
		v.0
	}
}

impl From<String> for VarValue {
	fn from(s: String) -> Self {
		Self(s)
	}
}

impl From<&str> for VarValue {
	fn from(s: &str) -> Self {
		Self(s.into())
	}
}

impl std::ops::Deref for VarValue {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl PartialEq for VarValue {
	fn eq(&self, other: &Self) -> bool {
		self.0.replace('_', "") == other.0.replace('_', "")
	}
}

/// Builds a [`Term::Value`] from a [`u128`].
#[macro_export]
macro_rules! scalar {
	($a:expr) => {
		$crate::term::Term::Scalar($a as u128)
	};
}

/// Builds a [`Term::Value`] from a [`u128`].
#[macro_export]
macro_rules! val {
	($a:expr) => {
		$crate::term::SimpleTerm::Value($a as u128)
	};
}

#[macro_export]
macro_rules! cval {
	($a:expr) => {
		$crate::term::Term::Value($a)
	};
}

/// Builds a [`Term::Var`] from a [`String`].
#[macro_export]
macro_rules! var {
	($a:expr) => {
		$crate::term::SimpleTerm::Var($a.into())
	};
}

/// Builds a [`Term::Var`] from a [`String`].
#[macro_export]
macro_rules! cvar {
	($a:expr) => {
		$crate::term::Term::Var($a.into())
	};
}

/// Builds a [`Term::Add`] from two [`Term`]s.
#[macro_export]
macro_rules! add {
	($a:expr, $b:expr) => {
		$crate::term::SimpleTerm::Add($a.into(), $b.into())
	};
}

/// Builds a [`Term::Add`] from two [`Term`]s.
#[macro_export]
macro_rules! cadd {
	($a:expr, $b:expr) => {
		$crate::term::ChromaticTerm::Add($a.into(), $b.into())
	};
}

/// Builds a [`Term::Mul`] from two [`Term`]s.
#[macro_export]
macro_rules! mul {
	($a:expr, $b:expr) => {
		$crate::term::SimpleTerm::Mul($a.into(), $b.into())
	};
}

/// Builds a [`Term::Mul`] from two [`Term`]s.
#[macro_export]
macro_rules! cmul {
	($a:expr, $b:expr) => {
		$crate::term::ChromaticTerm::Mul($a.into(), $b.into())
	};
}

impl SimpleTerm {
	/// Evaluates the term within the given scope to a concrete value.
	pub fn eval(&self, ctx: &crate::scope::SimpleScope) -> Result<u128, String> {
		match self {
			Self::Value(x) => Ok(*x),
			Self::Scalar(x) => Ok(*x),
			Self::Add(x, y) => Ok(x.eval(ctx)? + y.eval(ctx)?),
			Self::Mul(x, y) => Ok(x.eval(ctx)? * y.eval(ctx)?),
			Self::Var(x) =>
				if let Some(var) = ctx.get(x) {
					var.eval(ctx)
				} else {
					Err(format!("Variable '{}' not found", x.deref()))
				},
		}
	}

	pub fn into_chromatic(self, unit: crate::Dimension) -> ChromaticTerm {
		match self {
			Self::Value(x) | Self::Scalar(x) =>
				ChromaticTerm::Value(Self::scalar_into_term(x, unit)),
			Self::Add(x, y) => ChromaticTerm::Add(
				Box::new(x.into_chromatic(unit)),
				Box::new(y.into_chromatic(unit)),
			),
			Self::Mul(x, y) => ChromaticTerm::Mul(
				Box::new(x.into_chromatic(unit)),
				Box::new(y.into_chromatic(unit)),
			),
			Self::Var(x) => ChromaticTerm::Var(x),
		}
	}

	fn scalar_into_term(s: u128, unit: crate::Dimension) -> Weight {
		match unit {
			crate::Dimension::Time => Weight { time: s, proof: 0 },
			crate::Dimension::Proof => Weight { proof: s, time: 0 },
		}
	}
}

impl<T> Term<T>
where
	T: Clone + core::fmt::Display + One + Zero + PartialEq + Eq + ValueFormatter,
{
	pub fn is_const_zero(&self) -> bool {
		match self {
			Self::Value(x) => x == &T::zero(),
			Self::Scalar(x) => *x == 0,
			_ => false,
		}
	}

	pub fn is_const_one(&self) -> bool {
		match self {
			Self::Value(x) => x == &T::one(),
			Self::Scalar(x) => *x == 1,
			_ => false,
		}
	}

	/// Returns the variables of the term that are not part of [`crate::scope::Scope`].
	///
	/// Lambda calculus calls such a variable *free*.
	/// This is the inverse of [`Self::bound_vars`].
	pub fn free_vars(&self, scope: &Scope<Term<T>>) -> Set<String> {
		match self {
			Self::Var(var) if scope.get(var).is_some() => Set::default(),
			Self::Var(var) => Set::from([var.clone().into()]),
			Self::Scalar(_) => Set::default(),
			Self::Value(_) => Set::default(),
			Self::Mul(l, r) | Self::Add(l, r) =>
				l.free_vars(scope).union(&r.free_vars(scope)).cloned().collect(),
		}
	}

	/// Returns the variables of the term that are part of [`crate::scope::Scope`].
	///
	/// Lambda calculus calls such a variable *bound*.
	/// This is the inverse of [`Self::free_vars`].
	pub fn bound_vars(&self, scope: &Scope<Term<T>>) -> Set<String> {
		match self {
			Self::Var(var) if scope.get(var).is_some() => Set::from([var.clone().into()]),
			Self::Var(_var) => Set::default(),
			Self::Scalar(_) => Set::default(),
			Self::Value(_) => Set::default(),
			Self::Mul(l, r) | Self::Add(l, r) =>
				l.bound_vars(scope).union(&r.bound_vars(scope)).cloned().collect(),
		}
	}

	pub fn fmt_equation(&self, scope: &Scope<Term<T>>) -> String {
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

	pub fn into_substituted(self, var: &str, term: &Term<T>) -> Self {
		let mut s = self;
		s.substitute(var, term);
		s
	}

	pub fn substitute(&mut self, var: &str, term: &Term<T>) {
		match self {
			Self::Var(v) if v.0 == var => *self = term.clone(),
			Self::Var(_) => {},
			Self::Scalar(_) => {},
			Self::Value(_) => {},
			Self::Mul(l, r) | Self::Add(l, r) => {
				l.substitute(var, term);
				r.substitute(var, term);
			},
		}
	}

	fn fmt_with_bracket(&self, has_bracket: bool) -> String {
		self.maybe_fmt_with_bracket(has_bracket).unwrap_or("0".to_string())
	}

	fn maybe_fmt_with_bracket(&self, has_bracket: bool) -> Option<String> {
		match self {
			Self::Mul(l, r) => {
				// Omit `1 *` and `* 1`.
				if l.is_const_one() {
					r.maybe_fmt_with_bracket(has_bracket)
				} else if r.is_const_one() {
					l.maybe_fmt_with_bracket(has_bracket)
				} else if r.is_const_zero() || l.is_const_zero() {
					None
				} else {
					match (l.maybe_fmt_with_bracket(false), r.maybe_fmt_with_bracket(false)) {
						(Some(l), Some(r)) => Some(format!("{} * {}", l, r)),
						(Some(l), None) => Some(l),
						(None, Some(r)) => Some(r),
						(None, None) => None,
					}
				}
			},
			Self::Add(l, r) => {
				// Omit `0 +` and `+ 0`.
				if l.is_const_zero() && r.is_const_zero() {
					None
				} else if l.is_const_zero() {
					r.maybe_fmt_with_bracket(has_bracket)
				} else if r.is_const_zero() {
					l.maybe_fmt_with_bracket(has_bracket)
				} else if has_bracket {
					match (l.maybe_fmt_with_bracket(true), r.maybe_fmt_with_bracket(true)) {
						(Some(l), Some(r)) => Some(format!("{} + {}", l, r)),
						(Some(l), None) => Some(l),
						(None, Some(r)) => Some(r),
						(None, None) => None,
					}
				} else {
					match (l.maybe_fmt_with_bracket(true), r.maybe_fmt_with_bracket(true)) {
						(Some(l), Some(r)) => Some(format!("({} + {})", l, r)),
						(Some(l), None) => Some(l),
						(None, Some(r)) => Some(r),
						(None, None) => None,
					}
				}
			},
			Self::Value(val) => Some(val.format_scalar()),
			Self::Scalar(val) => Some(crate::Dimension::fmt_scalar(*val)),
			Self::Var(var) => Some(var.clone().into()),
		}
	}

	pub fn visit<F, R>(&self, f: &mut F) -> Result<Vec<R>, String>
	where
		F: FnMut(&Self) -> Result<R, String>,
	{
		let mut res = Vec::<R>::new();
		res.push(f(self)?);

		match self {
			v @ Self::Value(_) => Ok(vec![f(v)?]),
			v @ Self::Scalar(_) => Ok(vec![f(v)?]),
			v @ Self::Var(_) => Ok(vec![f(v)?]),
			Self::Add(l, r) | Self::Mul(l, r) => {
				res.append(&mut l.visit(f)?);
				res.append(&mut r.visit(f)?);
				Ok(res)
			},
		}
	}

	/// Returns the largest pre-factor of the variable in the term.
	pub fn find_largest_factor(&self, var: &str) -> Option<u128> {
		self.visit::<_, Option<u128>>(&mut |t| {
			if let Term::<T>::Mul(l, r) = t {
				if r.as_var() == Some(var) && l.as_scalar().is_some() {
					return Ok(Some(l.as_scalar().unwrap()))
				}
				if l.as_var() == Some(var) && r.as_scalar().is_some() {
					return Ok(Some(r.as_scalar().unwrap()))
				}
			}
			Ok(None)
		})
		.unwrap()
		.into_iter()
		.flatten()
		.max()
	}

	pub fn as_scalar(&self) -> Option<u128> {
		match self {
			Self::Scalar(val) => Some(*val),
			_ => None,
		}
	}

	pub fn as_var(&self) -> Option<&str> {
		match self {
			Self::Var(var) => Some(var),
			_ => None,
		}
	}
}

impl ChromaticTerm {
	/// Evaluates the term within the given scope to a concrete value.
	pub fn eval(&self, ctx: &crate::scope::ChromaticScope) -> Result<Weight, String> {
		match self {
			Self::Value(x) => Ok(x.clone()),
			Self::Scalar(_) => unreachable!("Scalars cannot be evaluated; qed"),
			Self::Add(x, y) => {
				let (a, b) = x.eval(ctx)?.into();
				let (m, n) = y.eval(ctx)?.into();
				Ok((a + m, b + n).into())
			},
			Self::Mul(x, y) => match (x.as_ref(), y.as_ref()) {
				(Self::Scalar(x), y) => {
					let (a, b) = y.eval(ctx)?.into();
					Ok((*x * a, *x * b).into())
				},
				(x, Self::Scalar(y)) => {
					let (a, b) = x.eval(ctx)?.into();
					Ok((*y * a, *y * b).into())
				},
				(Self::Var(x), y) => match ctx.get(x) {
					Some(Self::Scalar(x)) => Ok(y.eval(ctx)?.mul_scalar(x)),
					Some(_) => Err(format!("Variable '{}' is not a scalar", x.deref())),
					None => Err(format!("Variable '{}' not found", x.deref())),
				},
				(x, Self::Var(y)) => match ctx.get(y) {
					Some(Self::Scalar(y)) => Ok(x.eval(ctx)?.mul_scalar(y)),
					Some(_) => Err(format!("Variable '{}' is not a scalar", y.deref())),
					None => Err(format!("Variable '{}' not found", y.deref())),
				},
				_ => unreachable!("Cannot multiply two terms; qed"),
			},
			Self::Var(x) =>
				if let Some(var) = ctx.get(x) {
					var.eval(ctx)
				} else {
					Err(format!("Variable '{}' not found", x.deref()))
				},
		}
	}

	pub fn simplify(&self, unit: crate::Dimension) -> Result<SimpleTerm, String> {
		self.for_values(|t| match t {
			Self::Value(Weight { time, .. }) if unit == crate::Dimension::Time =>
				Ok(SimpleTerm::Value(*time)),
			Self::Value(Weight { proof, .. }) if unit == crate::Dimension::Proof =>
				Ok(SimpleTerm::Value(*proof)),
			Self::Scalar(val) => Ok(SimpleTerm::Scalar(*val)),
			Self::Var(var) => Ok(SimpleTerm::Var(var.clone())),
			_ => unreachable!(),
		})
	}

	pub fn for_values<F>(&self, f: F) -> Result<SimpleTerm, String>
	where
		F: Fn(&Self) -> Result<SimpleTerm, String> + Clone,
	{
		match self {
			v @ Self::Value(_) | v @ Self::Scalar(_) | v @ Self::Var(_) => f(v),
			Self::Mul(l, r) => Ok(SimpleTerm::Mul(
				l.for_values::<F>(f.clone())?.into(),
				r.for_values::<F>(f)?.into(),
			)),
			Self::Add(l, r) => Ok(SimpleTerm::Add(
				l.for_values::<F>(f.clone())?.into(),
				r.for_values::<F>(f)?.into(),
			)),
		}
	}

	/// Splice orthogonal weight terms together so that they produce a sum.
	pub fn splice_add(self, other: Self) -> Self {
		match (self, other) {
			(Self::Add(t1, p1), Self::Add(t2, p2)) =>
				Self::Add(Box::new(t1.splice_add(*t2)), Box::new(p1.splice_add(*p2))),
			(Self::Value(x), Self::Value(y)) => {
				// check for orthogonality
				if x.time == 0 && y.proof == 0 {
					Self::Value(Weight { time: y.time, proof: x.proof })
				} else if x.proof == 0 && y.time == 0 {
					Self::Value(Weight { time: x.time, proof: y.proof })
				} else {
					Self::Add(Box::new(Self::Value(x)), Box::new(Self::Value(y)))
				}
			},
			(s, o) => Self::Add(Box::new(s), Box::new(o)),
		}
	}
}

impl<T> fmt::Display for Term<T>
where
	T: Clone + core::fmt::Display + One + Zero + PartialEq + Eq + ValueFormatter,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.fmt_with_bracket(true))
	}
}

impl TryInto<SimpleTerm> for &ExprBinary {
	type Error = String;

	fn try_into(self) -> Result<SimpleTerm, Self::Error> {
		let left = crate::parse::pallet::parse_scalar_expression(&self.left)?.into();
		let right = crate::parse::pallet::parse_scalar_expression(&self.right)?.into();

		let term = match self.op {
			BinOp::Mul(_) => SimpleTerm::Mul(left, right),
			BinOp::Add(_) => SimpleTerm::Add(left, right),
			_ => return Err("Unexpected operator".into()),
		};
		Ok(term)
	}
}
