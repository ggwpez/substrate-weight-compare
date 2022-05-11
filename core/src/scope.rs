//! Provides a scope for evaluating [`Term`]s.

use crate::{term::Term, val, WEIGHT_PER_NANOS};
use std::collections::BTreeMap as Map;

pub const STORAGE_READ_VAR: &str = "READ";
pub const STORAGE_WRITE_VAR: &str = "WRITE";

/// A scope maps the constants to their values for [`Term::eval`].
pub trait Scope {
	fn get(&self, name: &str) -> Option<Term>;
	fn put_var(&mut self, name: &str, value: Term);
}

#[derive(Clone)]
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

	pub fn put_var(&mut self, name: &str, value: Term) {
		self.vars.insert(name.into(), value);
	}

	pub fn with_var(&self, name: &str, value: Term) -> Self {
		let mut ret = self.clone();
		ret.vars.insert(name.into(), value);
		ret
	}

	pub fn with_storage_weights(self, read: Term, write: Term) -> Self {
		self.with_var(STORAGE_READ_VAR, read).with_var(STORAGE_WRITE_VAR, write)
	}
}

impl Scope for BasicScope {
	fn get(&self, name: &str) -> Option<Term> {
		self.vars.get(&name.to_string()).cloned()
	}

	fn put_var(&mut self, name: &str, value: Term) {
		self.vars.insert(name.to_string(), value);
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

	/// Does nothing.
	fn put_var(&mut self, _name: &str, _value: Term) {}
}
