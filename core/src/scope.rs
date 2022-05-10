//! Provides a scope for evaluating [`Term`]s.

use crate::{term::Term, val};
use std::collections::BTreeMap as Map;

/// Hardcoded to avoid frame_support as dependency…
const WEIGHT_PER_NANOS: u128 = 1_000;

/// A scope maps the constants to their values for [`Term::eval`].
pub trait Scope {
	fn get(&self, name: &str) -> Option<Term>;

	fn read(&self) -> u128;
	fn write(&self) -> u128;
}

pub struct BasicScope {
	vars: Map<String, Term>,
}

impl BasicScope {
	pub fn empty() -> Self {
		Self { vars: Map::default() }
	}

	pub fn from_substrate() -> Self {
		(Self { vars: Map::default() })
			.with_var("WEIGHT_PER_NANOS", val!(WEIGHT_PER_NANOS))
			.with_var("constants::WEIGHT_PER_NANOS", val!(WEIGHT_PER_NANOS))
	}

	pub fn with_var(mut self, name: &str, value: Term) -> Self {
		assert!(!self.vars.contains_key(&name.to_string()), "Overwriting variable: {}", name);

		self.vars.insert(name.into(), value);
		self
	}
}

impl Scope for BasicScope {
	fn get(&self, name: &str) -> Option<Term> {
		self.vars.get(name.into()).cloned()
	}

	fn read(&self) -> u128 {
		if let Some(Term::Value(read)) = self.get("T::DbWeights::get().read()") {
			read
		} else {
			panic!("Unknown storage read")
		}
	}

	fn write(&self) -> u128 {
		if let Some(Term::Value(write)) = self.get("T::DbWeights::get().write()") {
			write
		} else {
			panic!("Unknown storage write")
		}
	}
}

/// A mocked scope for testing that returns some hardcoded values.
#[derive(Default)]
pub struct MockedScope {}

impl Scope for MockedScope {
	/// Returns 7 for all variables.
	fn get(&self, _: &str) -> Option<Term> {
		Some(Term::Value(7))
	}

	/// Returns 50.
	fn read(&self) -> u128 {
		50
	}

	/// Returns 100.
	fn write(&self) -> u128 {
		100
	}
}
