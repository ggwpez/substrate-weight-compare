//! Provides a scope for evaluating [`Term`]s.

use crate::{term::Term, val, WEIGHT_PER_NANOS};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;

pub const STORAGE_READ_VAR: &str = "READ";
pub const STORAGE_WRITE_VAR: &str = "WRITE";

#[derive(Clone, Serialize, Deserialize)]
pub struct Scope {
	vars: Map<String, Term>,
}

impl Scope {
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

	pub fn get(&self, name: &str) -> Option<Term> {
		self.vars.get(&name.to_string()).cloned()
	}

	pub fn merge(self, other: Self) -> Self {
		Self { vars: self.vars.into_iter().chain(other.vars).collect() }
	}

	pub fn extend(&mut self, other: Self) {
		self.vars.extend(other.vars);
	}

	pub fn len(&self) -> usize {
		self.vars.len()
	}

	pub fn is_empty(&self) -> bool {
		self.vars.is_empty()
	}
}

use std::fmt::Debug;
impl Debug for Scope {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = self
			.vars
			.iter()
			.map(|(k, v)| format!("{} = {}", k, v))
			.collect::<Vec<_>>()
			.join(", ");
		write!(f, "{}", s)
	}
}
