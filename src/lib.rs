#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::*;
use std::env::{current_dir, set_current_dir};

/*
** Test
*/

#[cfg(test)]
mod tests {
	use std::fs::{File, remove_file};
	use std::path::Path;
	use super::*;
	#[test]
	fn find_fail()
	{
		let _  = remove_file(Path::new("recipe.txt"));
		assert_eq!(super::Recipe::find(), None);
	}
	#[test]
	fn find_success()
	{
		let _ = File::create(Path::new("recipe.txt"));
		assert_ne!(super::Recipe::find(), None);
		let _ = remove_file(Path::new("recipe.txt"));
	}
	#[test]
	fn basic_parse()
	{
		let mut f = File::create(Path::new("recipe.txt")).unwrap();
		let mut rec: Recipe = Recipe::new();

		let _ = write!(f, "{}", 
			  "executable test\n".to_string()
			+ "    file.c2\n"
			+ "end\n"
		);

		rec.read_errors(true);
		assert_eq!(rec.ok, true);
		assert_eq!(rec.targets[0].name, "test".to_string());
		assert_eq!(rec.targets[0].kind, TargetType::Executable);
		assert_eq!(rec.targets[0].files.len(), 1);
		let _ = remove_file(Path::new("recipe.txt"));
	}
}

/*
** Types
*/

#[derive(Debug)]
pub struct Recipe
{
	pub ok: bool,
	pub path: PathBuf,
	pub target_count: u64,
	pub targets: Vec<Target>
}

#[derive(Clone, Debug)]
pub struct Target
{
	pub name: String,
	pub kind: TargetType,
	pub files: Vec<String>,
	pub options: TargetOptions,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TargetType
{
	SharedLib,
	StaticLib,
	Executable,
	Temporary, //used during declaration
}

#[derive(Clone, Debug)]
pub enum Use
{
	Static,
	Dynamic,
}

pub enum ReadState
{
	Start,
	InsideTarget,
}

#[derive(Clone, Debug)]
pub struct TargetOptions
{
	pub deps: bool,
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
			ok: false,
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

	//TODO
	fn read(&mut self)
	{
		self.read_errors(false);
	}

	fn read_errors(&mut self, errors: bool)
	{
		self.path = match Recipe::find()
		{
			Some(p) => PathBuf::from(p), 
			None    =>
			{	
				if errors {println!("error: recipe file not found in current path")};
				return;
			} 
		};
		let mut recipe_file = match File::open(&self.path)
		{
			Ok(f) => f,
			Err(_) =>
			{
				if errors {println!("error: could not open recipe file")};
				return;
			}
		};

		let mut contents = String::new();
		recipe_file.read_to_string(&mut contents).unwrap();

		let mut target = Target::new();
		let mut state = ReadState::Start;
		let mut line_number = 0;
		for line in contents.lines()
		{
			line_number += 1;
			let mut tokens = line.split_whitespace();
			match state
			{
				ReadState::Start => match tokens.next()
				{
					Some(x) => match x
					{
						"executable" =>
						{
							target.kind = TargetType::Executable;
							target.name = match tokens.next()
							{
								Some(s) => s.to_string(),
								None =>
								{
									if errors
										{println!("error: expected target identifier at line {}", line_number);}
									return;
								}
							};
							state = ReadState::InsideTarget;
						}
						"lib" =>
						{
							target.name = match tokens.next()
							{
								Some(s) => s.to_string(),
								None =>
								{
									if errors
										{println!("error: expected target identifier at line {}", line_number);}
									return;
								}
							};
							target.kind = match tokens.next()
							{
								Some(s) => match s
								{
									"shared" => TargetType::SharedLib,
									"static" => TargetType::StaticLib,
									x =>
									{
										if errors
											{println!("error: uknown library type '{}' at line {}", x, line_number)}
										return;
									} 
								},
								None =>
								{
									if errors
										{println!("error: expected a library type at line {}", line_number)}
									return ;
								}
							};
							state = ReadState::InsideTarget;
						}
						x =>
						{
							if errors
								{println!("error: unknown target type '{}' at line {}", x, line_number);}
							return;
						}
					},
					None => {}
				},
				ReadState::InsideTarget => match tokens.next()
				{
					Some(s) => match s
					{
						"end" => { self.targets.push(target.clone()); target = Target::new(); state = ReadState::Start; },
						"$refs" => target.options.refs = true,
						"$deps" => target.options.deps = true,
						"$nolibc" => target.options.nolibc = true,
						"$generate_ir" => target.options.generate_ir = true,
						"$generate_c" => target.options.generate_c = true,
						"$warnings" => loop
						{
							match tokens.next() { Some(p) => target.options.warnings.push(p.to_string()), 
												  None => break }
						},
						"$export" => loop
						{
						    match tokens.next() { Some(p) => target.options.export.push(p.to_string()),
						    					  None => break }
						},
						"$config" => loop
						{
							match tokens.next() { Some(p) => target.options.config.push(p.to_string()), 
												  None => break }
						},
						"$use" => match tokens.next()
						{
							Some(name) =>
							{
								match tokens.next()
								{
									Some(use_type) => match use_type
									{
										"static" => target.options.lib_use.push((name.to_string(), Use::Static)),
										"dynamic" => target.options.lib_use.push((name.to_string(), Use::Dynamic)),
										x =>
										{
											if errors
												{println!("error: unknown library type '{}' at line {}", x, line_number);}
											return;
										}
									},
									None =>
									{
										if errors
											{println!("error: missing library type after '{}' at line {}", name, line_number);}
										return;
									}
								}
							},
							None =>
							{
								if errors
									{println!("error: missing library name at line {}", line_number);}
								return;
							}
						},
						x => if !x.starts_with('$') {target.files.push(x.to_string())} else
						{
							if errors
								{println!("error: unknown option '{}' at line {}", x, line_number);}
							return;
						}
					},
					None => {}
				}
			}
		}
		self.ok = true;
	}

	fn chdir(&self)
	{
		let _ = set_current_dir(Path::new(&self.path));
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
			deps: false,
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
