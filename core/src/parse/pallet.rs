use crate::{reads, writes, ExtrinsicName, PalletName};

use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};
use syn::{
	punctuated::Punctuated, Attribute, Expr, ExprCall, ExprMethodCall, ImplItem, ImplItemMethod,
	Item, Lit, ReturnType, Stmt, Token, Type,
};

use crate::{
	mul,
	parse::{path_to_string, PathStripping},
	term::Term,
};

pub type Result<T> = std::result::Result<T, String>;

pub type ComponentName = String;

/// Inclusive range of a component.
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct ComponentRange {
	pub min: u32,
	pub max: u32,
}
pub type ComponentRanges = HashMap<ComponentName, ComponentRange>;

#[derive(Clone, Debug, PartialEq)]
pub struct Extrinsic {
	pub name: ExtrinsicName,
	pub pallet: PalletName,

	pub term: Term,
	/// Min and max value that each weight component can have.
	pub comp_ranges: Option<ComponentRanges>,
}

pub fn parse_file_in_repo(repo: &Path, file: &Path) -> Result<Vec<Extrinsic>> {
	let content = super::read_file(file)?;
	let name = PathStripping::RepoRelative.strip(repo, file);
	parse_content(name, content).map_err(|e| format!("{}: {}", file.display(), e))
}

pub fn parse_file(file: &Path) -> Result<Vec<Extrinsic>> {
	let content = super::read_file(file)?;
	let name = PathStripping::FileName.strip(Path::new("."), file);
	parse_content(name, content).map_err(|e| format!("{}: {}", file.display(), e))
}

pub fn parse_files_in_repo(repo: &Path, paths: &[PathBuf]) -> Result<Vec<Extrinsic>> {
	let mut res = Vec::new();
	for path in paths {
		res.extend(parse_file_in_repo(repo, path)?);
	}
	Ok(res)
}

pub fn parse_files(paths: &[PathBuf]) -> Result<Vec<Extrinsic>> {
	let mut res = Vec::new();
	for path in paths {
		res.extend(parse_file(path)?);
	}
	Ok(res)
}

pub fn try_parse_files_in_repo(repo: &Path, paths: &[PathBuf]) -> Vec<Extrinsic> {
	let mut res = Vec::new();
	for path in paths {
		if let Ok(parsed) = parse_file_in_repo(repo, path) {
			res.extend(parsed);
		}
	}
	res
}

pub fn try_parse_files(paths: &[PathBuf]) -> Vec<Extrinsic> {
	let mut res = Vec::new();
	for path in paths {
		if let Ok(parsed) = parse_file(path) {
			res.extend(parsed);
		}
	}
	res
}

pub fn parse_content(pallet: PalletName, content: String) -> Result<Vec<Extrinsic>> {
	let ast = syn::parse_file(&content).map_err(|e| format!("syn refused to parse: {}", e))?;
	for item in ast.items {
		if let Ok(weights) = handle_item(pallet.clone(), &item) {
			return Ok(weights)
		}
	}
	log::warn!("Could not find a weight implementation in {}", &pallet);
	Err("Could not find a weight implementation in the passed file".into())
}

pub(crate) fn handle_item(pallet: PalletName, item: &Item) -> Result<Vec<Extrinsic>> {
	match item {
		Item::Impl(imp) => {
			match imp.self_ty.as_ref() {
				// TODO handle both () and non () since ComposableFI uses ().
				Type::Tuple(t) => {
					if !t.elems.is_empty() {
						// The substrate template contains the weight info twice.
						// By skipping the not `impl ()` we ensure to parse it only once.
						return Err("Skipped ()".into())
					}
				},
				Type::Path(p) => {
					if p.path.leading_colon.is_some() {
						return Err("Skipped fn: impl leading color".into())
					}
					if p.path.segments.len() != 1 {
						return Err("Skipped fn: impl path segment len".into())
					}
					if let Some(last) = p.path.segments.last() {
						let name = last.ident.to_string();
						if name != "WeightInfo" && name != "SubstrateWeight" {
							return Err("Skipped fn: impl name last".into())
						}
					} else {
						return Err("Skipped fn: impl name segments".into())
					}
				},
				_ => return Err("Skipped fn: impl type".into()),
			}
			// TODO validate the trait type.
			let mut weights = Vec::new();
			for f in &imp.items {
				if let ImplItem::Method(m) = f {
					let (ext_name, term, comp_ranges) = handle_method(m)?;

					weights.push(Extrinsic {
						name: ext_name,
						pallet: pallet.clone(),
						term,
						comp_ranges,
					});
				}
			}
			if weights.is_empty() {
				Err("No weight functions found in trait impl".into())
			} else {
				Ok(weights)
			}
		},
		_ => Err("No weight trait impl found".into()),
	}
}

/// Parses range component attributes.
///
/// Returns `Ok(None)` if the attribute is was not detected.
/// Returns `Err(e)` if the attribute was detected but is invalid.
///
/// This doc comment:
///   The range of component `c` is `[1_337, 2000]`.
/// would be parsed into:
///   ("c", (1_337, =2000))
fn parse_component_attr(attr: &Attribute) -> Result<Option<(ComponentName, ComponentRange)>> {
	lazy_static! {
		// TODO syn seems to put a ="…" around the comment.
		static ref REGEX: Regex = Regex::new(
			r#"[\w\s]*`(?P<component>\w+)`[\w\s]*`\[(?P<min>[\d_]+),\s*(?P<max>[\d_]+)\]`.*"#
		)
		.unwrap();
	}

	let input = attr.tokens.to_string();
	let caps = REGEX.captures(&input).expect("Regex is known good");
	if caps.is_none() {
		return Ok(None)
	}
	let caps = caps.unwrap();

	let component = caps.name("component").ok_or("Missing component name")?.as_str();
	let min: u32 = caps
		.name("min")
		.ok_or("Min value not found")?
		.as_str()
		.replace('_', "")
		.parse()
		.map_err(|e| format!("Could not parse min value: {:?}", e))?;
	let max: u32 = caps
		.name("max")
		.ok_or("Max value not found")?
		.as_str()
		.replace('_', "")
		.parse()
		.map_err(|e| format!("Could not parse max value: {:?}", e))?;
	// Sanity check
	if min > max {
		return Err("Min value is greater than max value".into())
	}
	Ok(Some((component.into(), ComponentRange { min, max })))
}

fn parse_component_attrs(attrs: &Vec<Attribute>) -> Result<Option<ComponentRanges>> {
	let mut res = HashMap::new();
	for attr in attrs {
		match parse_component_attr(attr) {
			Ok(Some((name, range))) => {
				res.insert(name.replace('_', ""), range);
			},
			Ok(None) => {
				// Some kind of other attribute that we ignore.
			},
			Err(e) => return Err(e),
		}
	}

	if res.is_empty() {
		Ok(None)
	} else {
		Ok(Some(res))
	}
}

fn handle_method(m: &ImplItemMethod) -> Result<(ExtrinsicName, Term, Option<ComponentRanges>)> {
	let name = m.sig.ident.to_string();
	// Check the return type to end with `Weight`.
	if let ReturnType::Type(_, i) = &m.sig.output {
		if let Type::Path(p) = i.as_ref() {
			let n = p.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
			if !n.ends_with("Weight") {
				return Err(format!("Skipped fn: {} not a weight", name))
			}
		} else {
			return Err(format!("Skipped fn: {} not a weight", name))
		}
	} else {
		return Err("Skipped fn: method return type".into())
	}
	if m.block.stmts.len() != 1 {
		return Err("There must be only one statement per weight function".into())
	}
	let stmt = m.block.stmts.first().unwrap();

	let weight = match stmt {
		Stmt::Expr(expr) => parse_expression(expr)?,
		_ => unreachable!("Expected expression"),
	};
	// We later on check that the number of weight components matches
	// the number of components in the term. This cannot be done here
	// as global constants could mess up the counting.
	let comp_ranges = parse_component_attrs(&m.attrs)?;

	Ok((name, weight, comp_ranges))
}

pub(crate) fn parse_expression(expr: &Expr) -> Result<Term> {
	match expr {
		Expr::Paren(expr) => parse_expression(&expr.expr),
		// TODO check cast
		Expr::Cast(cast) => parse_expression(&cast.expr),
		Expr::MethodCall(call) => parse_method_call(call),
		Expr::Lit(lit) => Ok(Term::Value(lit_to_value(&lit.lit))),
		Expr::Path(p) => {
			let ident = path_to_string(&p.path, Some("::"));
			Ok(Term::Var(ident.into()))
		},
		Expr::Call(call) => parse_call(call),
		_ => Err("Unexpected expression".into()),
	}
}

// Example: T::DbWeight::get()
fn validate_db_call(call: &Expr) -> Result<()> {
	match call {
		Expr::Call(call) => {
			validate_db_func(&call.func)?;
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
fn validate_db_func(func: &Expr) -> Result<()> {
	match &func {
		Expr::Path(p) => {
			let path = p
				.path
				.segments
				.iter()
				.map(|s| s.ident.to_string())
				.collect::<Vec<_>>()
				.join("::");
			if path != "T::DbWeight::get" &&
				!path.ends_with("RocksDbWeight::get") &&
				!path.ends_with("ParityDbWeight::get")
			{
				Err(format!("Unexpected DB path: {}", path))
			} else {
				Ok(())
			}
		},
		_ => Err("Unexpected DB func".into()),
	}
}

// V1.5 feature
fn parse_call(call: &ExprCall) -> Result<Term> {
	let name = function_name(call)?;
	if name.ends_with("from_ref_time") {
		parse_args(&call.args)
	} else {
		Err(format!("Unexpected call: {}", name))
	}
}

// Example: receiver.saturating_mul(5 as Weight)
fn parse_method_call(call: &ExprMethodCall) -> Result<Term> {
	let name: &str = &call.method.to_string();
	match name {
		"reads" => {
			// Can only be called on T::DbWeight::get()
			validate_db_call(&call.receiver)?;
			let reads = parse_args(&call.args)?;
			Ok(reads!(reads))
		},
		"writes" => {
			// Can only be called on T::DbWeight::get()
			validate_db_call(&call.receiver)?;
			let writes = parse_args(&call.args)?;
			Ok(writes!(writes))
		},
		"saturating_add" =>
			Ok(Term::Add(parse_expression(&call.receiver)?.into(), parse_args(&call.args)?.into())),
		"saturating_mul" => Ok(mul!(parse_expression(&call.receiver)?, parse_args(&call.args)?)),
		_ => Err(format!("Unknown function: {}", name)),
	}
}

fn parse_args(args: &Punctuated<Expr, Token![,]>) -> Result<Term> {
	if args.len() != 1 {
		return Err(format!("Expected one argument, got {}", args.len()))
	}
	let args = args.first().ok_or("Empty args")?;
	parse_expression(args)
}

pub(crate) fn lit_to_value(lit: &Lit) -> u128 {
	match lit {
		Lit::Int(i) => i.base10_digits().parse().expect("Lit must be a valid int; qed"),
		_ => unreachable!(),
	}
}

fn function_name(call: &ExprCall) -> Result<String> {
	match call.func.as_ref() {
		Expr::Path(p) => Ok(path_to_string(&p.path, Some("::"))),
		_ => Err("Unexpected function".into()),
	}
}
