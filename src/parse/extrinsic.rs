use log::debug;
use std::{io::Read, path::PathBuf};
use syn::{ExprCall, ExprMethodCall};

use crate::*;

pub type PalletName = String;
pub type ExtrinsicName = String;

/// Maps an Extrinsic in the form of (PalletName, ExtrinsicName) to its Weight.
///
/// NOTE: Uses a 2D map for prefix matching.
pub type ParsedFiles = Map<PalletName, Map<ExtrinsicName, WeightNs>>;
pub type ParsedExtrinsic = Map<ExtrinsicName, WeightNs>;

const LOG: &str = "ext-parser";

pub fn parse_files(paths: &Vec<PathBuf>, blacklist: &Vec<String>) -> Result<ParsedFiles, String> {
	let mut map = Map::new();
	'outer: for path in paths {
		for skip in blacklist {
			if path.to_string_lossy().to_string().ends_with(skip) {
				continue 'outer
			}
		}
		map.insert(file_of(path), parse_file(path)?);
	}
	Ok(map)
}

/// Strips the path and only returns the file name.
fn file_of(path: &Path) -> String {
	path.file_name().unwrap().to_str().unwrap().to_string()
}

/// Parse the `BlockExecutionWeight` from the given file.
pub fn block_weight(path: &PathBuf) {}

/// Parse the `ExtrinsicBaseWeight` from the given file.
pub fn extrinsic_weight(path: &PathBuf) {}

/// Parse DB read+write weights from the given file.
pub fn db_weight(path: &PathBuf) {}

/// Parse extrinsic weight from the given file.
pub fn parse_file(path: &PathBuf) -> Result<ParsedExtrinsic, String> {
	debug!(target: LOG, "Entering file: {}", path.to_string_lossy());
	let mut file = ::std::fs::File::open(&path).unwrap();
	let mut content = String::new();
	file.read_to_string(&mut content).unwrap();

	let ast = syn::parse_file(&content).unwrap();
	for item in ast.items {
		if let Some(weights) = handle_item(&item) {
			return Ok(weights)
		}
	}
	Err(format!(
		"Could not find weight implementation in the passed file: {}\n\
    Ensure that you are using the template from the Substrate .maintain folder.",
		path.display()
	))
}

fn handle_item(item: &Item) -> Option<Map<String, WeightNs>> {
	match item {
		Item::Impl(imp) => {
			match imp.self_ty.as_ref() {
				Type::Tuple(t) if t.elems.is_empty() => {
					debug!(target: LOG, "Skipped fn: impl tuple type empty");
					// The substrate template contains the weight info twice.
					// By skipping the `impl ()` we ensure to parse it only once.
					return None
				},
				Type::Path(p) => {
					if p.path.leading_colon.is_some() {
						debug!(target: LOG, "Skipped fn: impl leading color");
						return None
					}
					if p.path.segments.len() != 1 {
						debug!(target: LOG, "Skipped fn: impl path segment len");
						return None
					}
					if let Some(last) = p.path.segments.last() {
						let name = last.ident.to_string();
						if name != "WeightInfo" {
							debug!(target: LOG, "Skipped fn: impl name last: {}", name);
							return None
						}
						debug!(target: LOG, "Using fn: impl name: {}", name);
					} else {
						debug!(target: LOG, "Skipped fn: impl name segments");
						return None
					}
				},
				_ => {
					debug!(target: LOG, "Skipped fn: impl type");
					return None
				},
			}
			let mut weights = Map::new();
			for f in &imp.items {
				if let ImplItem::Method(m) = f {
					let (name, weight) = handle_method(m).unwrap();
					weights.insert(name, weight.eval(&MockedScope::default())); // FIXME
				}
			}
			Some(weights)
		},
		_ => None,
	}
}

fn handle_method(m: &ImplItemMethod) -> Result<(String, Term), String> {
	let name = m.sig.ident.to_string();
	debug!(target: LOG, "Enter function {}", name);
	assert_eq!(
		m.block.stmts.len(),
		1,
		"There must be only one statement per weight function: {}",
		name
	);
	let stmt = m.block.stmts.first().unwrap();

	let weight = match stmt {
		Stmt::Expr(expr) => parse_expression(expr)?,
		_ => unreachable!("Expected expression"),
	};
	Ok((name, weight))
}

use std::boxed::Box;

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
	Value(u128),
	Var(String),
	Read,
	Write,

	Add(Box<Term>, Box<Term>),
	Mul(Box<Term>, Box<Term>),
}

pub trait Scope {
	fn get(&self, name: &str) -> Option<Term>;
	fn read(&self) -> u128;
	fn write(&self) -> u128;
}

#[derive(Default)]
pub struct MockedScope {}

impl Scope for MockedScope {
	fn get(&self, _name: &str) -> Option<Term> {
		Some(val!(7))
	}

	fn read(&self) -> u128 {
		50
	}

	fn write(&self) -> u128 {
		100
	}
}

impl Term {
	pub fn new(value: u128) -> Self {
		Term::Value(value)
	}

	pub fn add(self, x: Self) -> Self {
		Term::Add(self.into(), x.into())
	}

	pub fn mul(self, x: Self) -> Self {
		Term::Mul(self.into(), x.into())
	}

	pub fn eval(self, ctx: &impl Scope) -> u128 {
		match self {
			Self::Value(x) => x,
			Self::Add(x, y) => x.eval(ctx) + y.eval(ctx),
			Self::Mul(x, y) => x.eval(ctx) * y.eval(ctx),
			Self::Read => ctx.read(),
			Self::Write => ctx.write(),
			Self::Var(x) => {
				let var = ctx.get(&x).expect(&format!("Variable '{}' not found", x));
				var.clone().eval(ctx)
			},
		}
	}
}

impl From<u128> for Term {
	fn from(x: u128) -> Self {
		Term::Value(x)
	}
}

pub(crate) fn parse_expression(expr: &Expr) -> Result<Term, String> {
	match expr {
		Expr::Paren(expr) => parse_expression(&expr.expr),
		// TODO check cast
		Expr::Cast(cast) => parse_expression(&cast.expr),
		Expr::MethodCall(call) => parse_method_call(call),
		Expr::Lit(lit) => Ok(Term::Value(lit_to_value(&lit.lit))),
		Expr::Path(p) => {
			if p.path.segments.len() != 1 {
				return Err("Unexpected path as weight constant".into())
			}
			let ident = p.path.segments.first().unwrap().ident.to_string();
			Ok(Term::Var(ident))
		},
		_ => Err("Unexpected expression".into()),
	}
}

// Example: T::DbWeight::get()
fn validate_db_call(call: &Expr) -> Result<(), String> {
	match call {
		Expr::Call(call) => {
			let _ = validate_db_func(&call.func)?;
			if !call.args.is_empty() {
				Err("Unexpected arguments".into())
			} else {
				Ok(())
			}
		},
		_ => Err("Unexpected DB call expression".into()),
	}
}

// example: T::DbWeight::get
fn validate_db_func(func: &Expr) -> Result<(), String> {
	match &func {
		Expr::Path(p) => {
			let path = p
				.path
				.segments
				.iter()
				.map(|s| s.ident.to_string())
				.collect::<Vec<_>>()
				.join("::");
			if path != "T::DbWeight::get" {
				Err(format!("Unexpected DB path: {}", path))
			} else {
				Ok(())
			}
		},
		_ => Err("Unexpected DB func".into()),
	}
}

// Example: receiver.saturating_mul(5 as Weight)
fn parse_method_call(call: &ExprMethodCall) -> Result<Term, String> {
	let name: &str = &call.method.to_string();
	match name {
		"reads" => {
			// Can only be called on T::DbWeight::get()
			validate_db_call(&call.receiver)?;
			let reads = parse_args(&call.args.iter().collect())?;
			Ok(reads!(reads))
		},
		"writes" => {
			// Can only be called on T::DbWeight::get()
			validate_db_call(&call.receiver)?;
			let writes = parse_args(&call.args.iter().collect())?;
			Ok(writes!(writes))
		},
		"saturating_add" => Ok(Term::Add(
			parse_expression(&call.receiver)?.into(),
			parse_args(&call.args.iter().collect())?.into(),
		)),
		"saturating_mul" => Ok(Term::Mul(
			parse_expression(&call.receiver)?.into(),
			parse_args(&call.args.iter().collect())?.into(),
		)),
		_ => Err(format!("Unknown function: {}", name)),
	}
}

fn parse_args(args: &Vec<&Expr>) -> Result<Term, String> {
	if args.len() != 1 {
		return Err(format!("Expected one argument, got {}", args.len()))
	}
	let args = *args.first().unwrap();
	parse_expression(&args)
}

pub(crate) fn lit_to_value(lit: &Lit) -> u128 {
	match lit {
		Lit::Int(i) => i.base10_digits().parse().unwrap(),
		_ => unreachable!(),
	}
}

#[macro_export]
macro_rules! val {
	($a:expr) => {
		Term::Value(($a as u128).into())
	};
}

#[macro_export]
macro_rules! add {
	($a:expr, $b:expr) => {
		Term::Add($a.into(), $b.into())
	};
}

#[macro_export]
macro_rules! mul {
	($a:expr, $b:expr) => {
		Term::Mul($a.into(), $b.into())
	};
}

#[macro_export]
macro_rules! reads {
	($a:expr) => {
		mul!($a, Term::Read)
	};
}

#[macro_export]
macro_rules! writes {
	($a:expr) => {
		mul!($a, Term::Write)
	};
}

#[macro_export]
macro_rules! var {
	($a:expr) => {
		Term::Var($a.into())
	};
}
