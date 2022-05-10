//! Compares [`Term`]s with each other.

use crate::term::Term;
use crate::scope::{BasicScope, Scope};

pub fn fn diff(f: &Term, g: &Term) {
	let scope = BasicScope::from_substrate();

	let fx = f.unbound_vars(&scope);
	let gx = f.unbound_vars(&scope);

	log::info!("f: {:?}", fx);
}
