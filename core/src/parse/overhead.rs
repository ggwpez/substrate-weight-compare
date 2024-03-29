use syn::ItemConst;

use crate::{
	parse::path_to_string,
	term::{ChromaticTerm, SimpleTerm},
	*,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Weight {
	BlockExecution(ChromaticTerm),
	ExtrinsicBase(ChromaticTerm),
}

pub fn parse_file(file: &Path) -> Result<Weight, String> {
	let content = super::read_file(file)?;
	parse_content(content)
}

pub fn parse_content(content: String) -> Result<Weight, String> {
	let ast = syn::parse_file(&content).map_err(|e| e.to_string())?;
	for item in ast.items {
		if let Ok(res) = handle_item(&item) {
			return Ok(res)
		}
	}
	Err("No Overhead weights found".to_string())
}

fn handle_item(item: &Item) -> Result<Weight, String> {
	match item {
		// The current Substrate template has a useless `constants` mod.
		Item::Mod(m) => {
			if m.ident == "constants" {
				if let Some((_, content)) = m.content.as_ref() {
					for item in content {
						let res = handle_item(item);
						// Ignore errors
						if res.is_ok() {
							return res
						}
					}
					return Err("Did not find parameter_types!".into())
				}
			}
			Err(format!("Unexpected module: {}", m.ident))
		},
		Item::Macro(m) => {
			let name = m.mac.path.segments.last();
			if name.unwrap().ident == "parameter_types" {
				parse_macro(m.mac.tokens.clone())
			} else {
				Err("Unexpected macro def".into())
			}
		},
		_ => Err("Could not find overhead weight in the file".into()),
	}
}

/// Handles the content of the `parameter_types!` macro.
///
/// Example:
/// ```nocompile
/// pub const BlockExecutionWeight: Weight = 5_481_991 * WEIGHT_PER_NANOS;
/// ```
fn parse_macro(tokens: proc_macro2::TokenStream) -> Result<Weight, String> {
	let def: ItemConst = syn::parse2(tokens).map_err(|e| e.to_string())?;
	let name = def.ident.to_string();

	let type_name = type_to_string(&def.ty, None)?;
	if type_name != "Weight" {
		return Err(format!("Unexpected const type: {}", type_name))
	}
	let weight: ChromaticTerm = match def.expr.as_ref() {
		Expr::Binary(bin) => {
			let simple: SimpleTerm = bin.try_into()?;
			Ok(simple.into_chromatic(crate::Dimension::Time))
		},
		e => super::pallet::parse_expression(e),
	}?;

	match name.as_str() {
		"BlockExecutionWeight" => Ok(Weight::BlockExecution(weight)),
		"ExtrinsicBaseWeight" => Ok(Weight::ExtrinsicBase(weight)),
		_ => Err(format!("Unexpected const name: {}", name)),
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
