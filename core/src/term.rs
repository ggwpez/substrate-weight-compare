//! Contains the [`Term`] which is used to express weight equations.

use syn::{BinOp, ExprBinary};

use crate::scope::Scope;

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
	StorageRead,
	StorageWrite,

	Add(Box<Term>, Box<Term>),
	Mul(Box<Term>, Box<Term>),
}

impl Term {
	/// Evaluates the term within the given scope to a concrete value.
	pub fn eval(self, ctx: &impl crate::scope::Scope) -> u128 {
		match self {
			Self::Value(x) => x,
			Self::Add(x, y) => x.eval(ctx) + y.eval(ctx),
			Self::Mul(x, y) => x.eval(ctx) * y.eval(ctx),
			Self::StorageRead => ctx.read(),
			Self::StorageWrite => ctx.write(),
			Self::Var(x) => {
				// TODO change to result
				let var = ctx.get(&x).unwrap_or_else(|| panic!("Variable '{}' not found", x));
				var.eval(ctx)
			},
		}
	}

	/// Returns the variables of the term that are not part of [`crate::scope::Scope`].
	pub fn unbound_vars(&self, scope: &impl Scope) -> Vec<String> {
		match self {
			Self::Var(var) if scope.get(var).is_some() => vec![],
			Self::Var(var) => vec![var.into()],

			Self::Value(_) | Self::StorageRead | Self::StorageWrite => vec![],
			Self::Mul(l, r) | Self::Add(l, r) =>
				[l.unbound_vars(scope), r.unbound_vars(scope)].concat(),
		}
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
