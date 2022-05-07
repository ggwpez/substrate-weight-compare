use super::extrinsic::Term;
use log::debug;
use std::path::PathBuf;
use syn::{BinOp, ExprStruct, ItemConst};

use crate::*;

const LOG: &str = "db-parser";

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Db {
	Parity,
	Rocks,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RWs {
	pub read: Term,
	pub write: Term,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DbWeights {
	pub db: Db,
	pub weights: RWs,
}

pub fn parse_file(file: &PathBuf) -> Result<DbWeights, String> {
	let content = super::read_file(file)?;
	parse_content(content)
}

pub fn parse_content(content: String) -> Result<DbWeights, String> {
	let ast = syn::parse_file(&content).map_err(err_to_str)?;
	for item in ast.items {
		if let Ok(res) = handle_item(&item) {
			return Ok(res)
		}
	}
	Err("No DB weights found".to_string())
}

fn handle_item(item: &Item) -> Result<DbWeights, String> {
	debug!(target: LOG, "Entering item");
	match item {
		// The current Substrate template has a useless `constants` mod.
		Item::Mod(m) => {
			debug!(target: LOG, "Entering module");
			if m.ident.to_string() == "constants" {
				if let Some((_, content)) = m.content.as_ref() {
					for item in content {
						let res = handle_item(&item);
						// Ignore errors
						if res.is_ok() {
							return res
						}
					}
					return Err("Did not find parameter_types!".into())
				}
			}
			Err(format!("Unexpected module: {}", m.ident.to_string()))
		},
		Item::Macro(m) => {
			let name = m.mac.path.segments.last();
			if name.unwrap().ident.to_string() == "parameter_types" {
				parse_macro(m.mac.tokens.clone())
			} else {
				Err("Unexpected macro def".into())
			}
		},
		_ => Err("Could not find DB weights in the file".into()),
	}
}

/// Handles the content of the `parameter_types!` macro.
///
/// Example:
/// ```nocompile
/// pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
/// 	read: 25_000 * constants::WEIGHT_PER_NANOS,
/// 	write: 100_000 * constants::WEIGHT_PER_NANOS,
/// };
/// ```
fn parse_macro(tokens: proc_macro2::TokenStream) -> Result<DbWeights, String> {
	let def: ItemConst = syn::parse2(tokens).map_err(err_to_str)?;
	let name = def.ident.to_string();

	let db = match name.as_str() {
		"RocksDbWeight" => Db::Rocks,
		"ParityDbWeight" => Db::Parity,
		_ => return Err(format!("Unexpected const name: {}", name)),
	};
	let type_name = type_to_string(&def.ty, None)?;
	if type_name != "RuntimeDbWeight" {
		return Err(format!("Unexpected const type: {}", type_name))
	}
	match def.expr.as_ref() {
		Expr::Struct(s) => {
			let weights = parse_runtime_db_weight(s)?;
			Ok(DbWeights { db, weights })
		},
		_ => Err("Unexpected const value".into()),
	}
}

fn parse_runtime_db_weight(expr: &ExprStruct) -> Result<RWs, String> {
	let name = path_to_string(&expr.path, None);
	if name != "RuntimeDbWeight" {
		return Err(format!("Unexpected struct name: {}", name))
	} else if expr.fields.len() != 2 {
		return Err("Unexpected struct fields".into())
	}
	let reads = expr
		.fields
		.iter()
		.find(|f| member_to_string(&f.member) == "read")
		.ok_or("No read field found")?;
	let writes = expr
		.fields
		.iter()
		.find(|f| member_to_string(&f.member) == "write")
		.ok_or("No write field found")?;

	let read = parse_expression(&reads.expr)?;
	let write = parse_expression(&writes.expr)?;

	Ok(RWs { read, write })
}

fn parse_expression(expr: &Expr) -> Result<Term, String> {
	match expr {
		Expr::Binary(bin) => {
			let left = parse_expression(&bin.left)?.into();
			let right = parse_expression(&bin.right)?.into();

			let term = match bin.op {
				BinOp::Mul(_) => Term::Mul(left, right),
				BinOp::Add(_) => Term::Add(left, right),
				_ => return Err("Unexpected operator".into()),
			};
			Ok(term)
		},
		Expr::Lit(lit) => Ok(Term::Value(super::extrinsic::lit_to_value(&lit.lit))),
		Expr::Path(p) => Ok(Term::Var(path_to_string(&p.path, Some("::")))),
		_ => Err("Unexpected expression".into()),
	}
}

/// Expects a path to a type and returns the type name.
fn type_to_string(p: &syn::Type, delimiter: Option<&str>) -> Result<String, String> {
	if let Type::Path(p) = p {
		Ok(path_to_string(&p.path, delimiter))
	} else {
		Err("Unexpected type".into())
	}
}

fn path_to_string(p: &syn::Path, delimiter: Option<&str>) -> String {
	p.segments
		.iter()
		.map(|s| s.ident.to_string())
		.collect::<Vec<_>>()
		.join(&delimiter.unwrap_or_default())
}

fn member_to_string(m: &syn::Member) -> String {
	match m {
		syn::Member::Named(ident) => ident.to_string(),
		_ => "".into(),
	}
}

fn err_to_str<E: std::string::ToString>(err: E) -> String {
	err.to_string()
}
