use crate::{reads, writes, ExtrinsicName, PalletName};

use std::path::{Path, PathBuf};
use syn::{
	punctuated::Punctuated, Expr, ExprMethodCall, ImplItem, ImplItemMethod, Item, Lit, ReturnType,
	Stmt, Token, Type,
};

use crate::{mul, term::Term};

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone)]
pub struct Extrinsic {
	pub name: ExtrinsicName,
	pub pallet: PalletName,

	pub term: Term,
}

pub fn parse_file(repo: &Path, file: &Path) -> Result<Vec<Extrinsic>> {
	let content = super::read_file(&repo.join(file))?;
	let name = file.strip_prefix(repo).unwrap_or(&file);
	parse_content(name.display().to_string(), content)
		.map_err(|e| format!("{}: {}", file.display(), e))
}

pub fn parse_files(repo: &Path, paths: &[PathBuf]) -> Result<Vec<Extrinsic>> {
	let mut map = Vec::new();
	for path in paths {
		map.extend(parse_file(repo, path)?);
	}
	Ok(map)
}

pub fn try_parse_files(repo: &Path, paths: &[PathBuf]) -> Vec<Extrinsic> {
	let mut map = Vec::new();
	for path in paths {
		if let Ok(res) = parse_file(repo, path) {
			map.extend(res);
		}
	}
	map
}

pub fn parse_content(pallet: PalletName, content: String) -> Result<Vec<Extrinsic>> {
	let ast = syn::parse_file(&content).map_err(|e| e.to_string())?;
	for item in ast.items {
		if let Ok(weights) = handle_item(pallet.clone(), &item) {
			return Ok(weights)
		}
	}
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
					let (ext_name, term) = handle_method(m)?;
					weights.push(Extrinsic { name: ext_name, pallet: pallet.clone(), term });
				}
			}
			Ok(weights)
		},
		_ => Err("No weight trait impl found".into()),
	}
}

fn handle_method(m: &ImplItemMethod) -> Result<(ExtrinsicName, Term)> {
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
	Ok((name, weight))
}

pub(crate) fn parse_expression(expr: &Expr) -> Result<Term> {
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
			let ident = p.path.segments.first().ok_or("Empty path")?.ident.to_string();
			Ok(Term::Var(ident))
		},
		_ => Err("Unexpected expression".into()),
	}
}

// Example: T::DbWeight::get()
fn validate_db_call(call: &Expr) -> Result<()> {
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
