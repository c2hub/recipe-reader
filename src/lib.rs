use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::*;
use std::env::{current_dir, set_current_dir};

/*
** Test
*/

#[cfg(test)]
mod tests {
	#[test]
	fn it_works()
	{

	}
}

/*
** Types
*/

pub struct Recipe
{
	pub path: PathBuf,
	pub target_count: u64,
	pub targets: Vec<Target>
}


pub struct Target
{
	pub name: String,
	pub kind: TargetType,
	pub files: Vec<String>,
	pub options: TargetOptions,
}

#[derive(Copy, Clone)]
pub enum TargetType
{
	SharedLib,
	StaticLib,
	Executable,
	Temporary, //Should never occur, used during declaration
}

#[derive(Clone)]
pub enum Use
{
	Static,
	Shared,
}

#[derive(Clone)]
pub struct TargetOptions
{
	pub refs: bool,
	pub nolibc: bool,
	pub generate_c: bool,
	pub generate_ir: bool,
	pub lib_use: Vec<(String, Use)>,
	pub export: Vec<String>,
	pub config: Vec<String>,
	pub warnings: Vec<String>,
}


/*
** Impls
*/

impl Recipe
{
	fn new() -> Recipe
	{
		Recipe
		{
			path: PathBuf::new(),
			target_count: 0,
			targets: Vec::new(),
		}
	}

	fn find() -> Option<String>
	{
		let cwd = current_dir().unwrap();
		let mut path = cwd.as_path();
		let mut recipe = path.join(Path::new("recipe.txt"));

		loop 
		{
			let recipe_f = File::open(&recipe);
			if recipe_f.is_err()
			{
				match path.parent()
				{
					Some(p) =>
					{
						path = p;
						recipe = path.join(Path::new("recipe.txt"));
					},
					None => return None
				}
			}
			else
			{
				return Some(recipe.into_os_string().into_string().unwrap()); 
			}
		}
	}
}

impl Target
{
	fn new() -> Target
	{
		Target
		{
			name: String::new(),
			kind: TargetType::Temporary,
			files: Vec::new(),
			options: TargetOptions::new()
		} 
	}
}

impl TargetOptions
{
	pub fn new() -> TargetOptions
	{
		TargetOptions
		{
			refs: false,
			nolibc: false,
			generate_c: false,
			generate_ir: false,
			lib_use: Vec::new(),
			export: Vec::new(),
			config: Vec::new(),
			warnings: Vec::new(),
		}
	}
}
