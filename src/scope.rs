//! Provides a scope for evaluating [`Term`]s.

use crate::term::Term;

/// A scope maps the constants to their values for [`Term::eval`].
pub trait Scope {
	fn get(&self, name: &str) -> Option<Term>;
	fn read(&self) -> u128;
	fn write(&self) -> u128;
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
