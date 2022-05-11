//! Compares [`Term`]s with each other.

use crate::term::Term;
use crate::scope::{BasicScope, Scope};

pub fn diff(f: &Term, g: &Term) {
	let scope = BasicScope::from_substrate();

	let fx = f.free_vars(&scope);
	let gx = f.free_vars(&scope);

	log::info!("f: {:?}", fx);
}
