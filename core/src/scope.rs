//! Provides a scope for evaluating [`Term`]s.

use crate::{
	term::{ChromaticTerm, SimpleTerm},
	WEIGHT_PER_NANOS,
};
use core::fmt::Display;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;

pub const STORAGE_READ_VAR: &str = "READ";
pub const STORAGE_WRITE_VAR: &str = "WRITE";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
#[cfg_attr(feature = "bloat", derive(Default))]
pub struct GenericScope<T> {
	vars: Map<String, T>,
}

pub type SimpleScope = GenericScope<SimpleTerm>;
pub type ChromaticScope = GenericScope<ChromaticTerm>;

impl SimpleScope {
	pub fn from_substrate() -> Self {
		(Self { vars: Map::default() })
			.with_var("WEIGHT_PER_NANOS", SimpleTerm::Scalar(WEIGHT_PER_NANOS))
			.with_var("constants::WEIGHT_PER_NANOS", SimpleTerm::Scalar(WEIGHT_PER_NANOS))
	}

	pub fn with_storage_weights(self, read: SimpleTerm, write: SimpleTerm) -> Self {
		self.with_var(STORAGE_READ_VAR, read).with_var(STORAGE_WRITE_VAR, write)
	}
}

impl<T> GenericScope<T>
where
	T: Clone,
{
	pub fn empty() -> Self {
		Self { vars: Map::default() }
	}

	pub fn put_var(&mut self, name: &str, value: T) {
		self.vars.insert(name.into(), value);
	}

	pub fn with_var(&self, name: &str, value: T) -> Self {
		let mut ret = self.clone();
		ret.vars.insert(name.into(), value);
		ret
	}

	pub fn get(&self, name: &str) -> Option<T> {
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

	pub fn as_vec(&self) -> Vec<(String, T)> {
		self.vars.clone().into_iter().collect()
	}
}

impl<T: Display> Display for GenericScope<T> {
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
