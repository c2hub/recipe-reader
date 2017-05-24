#![allow(dead_code)]
#![allow(unknown_lints)]
#![allow(cyclomatic_complexity)]
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::*;
use std::env::{current_dir, set_current_dir};

/*
** Tests
*/

#[cfg(test)]
mod tests;

/*
** Types
*/

#[derive(Debug, Default)]
pub struct Recipe
{
	pub ok: bool,
	pub path: PathBuf,
	pub target_count: u64,
	pub targets: Vec<Target>
}

#[derive(Clone, Debug, Default)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, Default)]
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

impl ToString for TargetType
{
	fn to_string(&self) -> String
	{
		match *self
		{
			TargetType::Executable => "executable".to_string(),
			TargetType::SharedLib => "shared".to_string(),
			TargetType::StaticLib => "static".to_string(),
			TargetType::Temporary => panic!("temporary target type is not allowed to be stringified"),
		}
	}
}

impl Default for TargetType
{
	fn default() -> TargetType
	{
		TargetType::Temporary
	}
}

impl ToString for Use
{
	fn to_string(&self) -> String
	{
		match *self
		{
			Use::Static => "static".to_string(),
			Use::Dynamic => "dynamic".to_string(),
		}
	}
}

impl Recipe
{
	pub fn new() -> Recipe
	{
		let mut temp: Recipe = Default::default();
		temp.ok = true;
		temp
	}

	pub fn find() -> Option<String>
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

	pub fn read(&mut self)
	{
		self.read_errors(false);
	}

	pub fn read_errors(&mut self, errors: bool)
	{
		self.ok = false;
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
			if line.starts_with('#') { continue; }
			let mut tokens = line.split_whitespace();
			match state
			{
				ReadState::Start => if let Some(x) = tokens.next()
				{
					match x
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

							//check for extra tokens
							if let Some(s) = tokens.next()
							{
								if errors
									{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
								return;
							}
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

							//check for extra tokens
							if let Some(s) = tokens.next()
							{
								if errors
									{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
								return;
							}
							state = ReadState::InsideTarget;
						}
						x =>
						{
							if errors
								{println!("error: unknown target type '{}' at line {}", x, line_number);}
							return;
						}
					}
				},
				ReadState::InsideTarget => if let Some(s) = tokens.next()
				{
					match s
					{
						"end" =>
						{
							self.targets.push(target.clone());
							self.target_count += 1;
							target = Target::new();
							state = ReadState::Start;

							//check for extra tokens
							if let Some(s) = tokens.next()
							{
								if errors
									{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
								return;
							}
						},
						"$refs" =>
						{
							target.options.refs = true;

							//check for extra tokens
							if let Some(s) = tokens.next()
							{
								if errors
									{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
								return;
							}
						}
						"$deps" =>
						{
							target.options.deps = true;

							//check for extra tokens
							if let Some(s) = tokens.next()
							{
								if errors
									{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
								return;
							}
						}
						"$nolibc" =>
						{
							target.options.nolibc = true;

							//check for extra tokens
							if let Some(s) = tokens.next()
							{
								if errors
									{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
								return;
							}
						}
						"$generate-ir" =>
						{
							target.options.generate_ir = true;

							//check for extra tokens
							if let Some(s) = tokens.next()
							{
								if errors
									{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
								return;
							}
						}
						"$generate-c" =>
						{
							target.options.generate_c = true;

							//check for extra tokens
							if let Some(s) = tokens.next()
							{
								if errors
									{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
								return;
							}
						}
						"$warnings" => while let Some(p) = tokens.next()
						{
							target.options.warnings.push(p.to_string());
						},
						"$export" => while let Some(p) = tokens.next()
						{
							target.options.export.push(p.to_string());
						},
						"$config" => while let Some(p) = tokens.next()
						{
							target.options.config.push(p.to_string()); 
						},
						"$use" => match tokens.next()
						{
							Some(name) =>
							{
								match tokens.next()
								{
									Some(use_type) => match use_type
									{
										"static" =>
										{
											if !target.options.lib_use.contains(&(name.to_string(), Use::Static))
												{target.options.lib_use.push((name.to_string(), Use::Static));}
											else
											{
												if errors
													{println!("error: duplicate library use '{}' at line {}", name, line_number);}
												return;
											}
										},
										"dynamic" =>
										{
											if !target.options.lib_use.contains(&(name.to_string(), Use::Dynamic))
												{target.options.lib_use.push((name.to_string(), Use::Dynamic));}
											else
											{
												if errors
													{println!("error: duplicate library use '{}' at line {}", name, line_number);}
												return;
											}
										},
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
						x =>
						{
							if !x.starts_with('$')
							{
								if !target.files.contains(&x.to_string())
									{target.files.push(x.to_string());}
								else
								{
									if errors
										{println!("error: duplicate file '{}' at line {}", x, line_number);}
									return;
								}
							}
							else
							{
								if errors
									{println!("error: unknown option '{}' at line {}", x, line_number);}
								return;
							}
						}
					}
				}
			}
		}
		self.ok = true;
	}

	pub fn write(&self)
	{
		self.write_path(&self.path);
	}

	pub fn write_path(&self, path: &PathBuf)
	{
		let mut f = File::create(path).unwrap();

		writeln!(f, "# this file is generated by recipe-reader, it might be overwritten\n")
			.expect("error: failed to write to recipe file");
		for trg_ptr in &self.targets
		{
			let trg = &(*trg_ptr);
			match trg.kind
			{
				TargetType::Executable => {writeln!(f, "executable {}", trg.name).expect("error: failed to write to recipe file");},
				TargetType::SharedLib | TargetType::StaticLib => {writeln!(f, "lib {} {}", trg.name, trg.kind.to_string()).expect("error: failed to write to recipe file");},
				TargetType::Temporary => panic!("error: temporary target type is invalid"),
			}

			if trg.options.generate_c 	{writeln!(f, "\t$generate-c").expect("error: failed to write to recipe file");}
			if trg.options.generate_ir 	{writeln!(f, "\t$generate-ir").expect("error: failed to write to recipe file");}
			if trg.options.nolibc 		{writeln!(f, "\t$nolibc").expect("error: failed to write to recipe file");}
			if trg.options.deps 		{writeln!(f, "\t$deps").expect("error: failed to write to recipe file");}
			if trg.options.refs 		{writeln!(f, "\t$refs").expect("error: failed to write to recipe file");}
			if !trg.options.export.is_empty()
			{
				write!(f, "\t$export").expect("error: failed to write to recipe file");
				for export in &trg.options.export
				{
					write!(f, " {}", export).expect("error: failed to write to recipe file");
				}
				write!(f, "\n").expect("error: failed to write to recipe file");
			}
			if !trg.options.config.is_empty()
			{
				write!(f, "\t$config").expect("error: failed to write to recipe file");
				for config in &trg.options.config
				{
					write!(f, " {}", config).expect("error: failed to write to recipe file");
				}
				write!(f, "\n").expect("error: failed to write to recipe file");
			}
			if !trg.options.warnings.is_empty()
			{
				write!(f, "\t$warnings").expect("error: failed to write to recipe file");
				for warning in &trg.options.warnings
				{
					write!(f, " {}", warning).expect("error: failed to write to recipe file");
				}
				write!(f, "\n").expect("error: failed to write to recipe file");
			}
			if !trg.options.lib_use.is_empty()
			{
				for lib_use in &trg.options.lib_use
				{
					writeln!(f, "\t$use {} {}", lib_use.0, lib_use.1.to_string()).expect("error: failed to write to recipe file");
				}
			}
			for file in &trg.files
			{
				writeln!(f, "\t{}", file).expect("error: failed to write to recipe file");
			}
			writeln!(f, "end\n").expect("error: failed to write to recipe file");
		}
	}

	pub fn chdir(&self)
	{
		set_current_dir(Path::new(&self.path)).expect("error: failed to chdir");
	}

	pub fn add_target(&mut self, trg: Target)
	{
		self.targets.push(trg);
		self.target_count += 1;
	}
}

impl Target
{
	fn new() -> Target { Default::default() }
}

impl TargetOptions
{
	pub fn new() -> TargetOptions
	{
		Default::default()
		/*TargetOptions
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
		}*/
	}
}
