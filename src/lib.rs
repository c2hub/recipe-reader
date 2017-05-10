#![allow(dead_code)]
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

impl ToString for TargetType
{
	fn to_string(&self) -> String
	{
		let ref me = *self;
		match me
		{
			&TargetType::Executable => return "executable".to_string(),
			&TargetType::SharedLib => return "shared".to_string(),
			&TargetType::StaticLib => return "static".to_string(),
			&TargetType::Temporary => panic!("temporary target type is not allowed to be stringified"),
		}
	}
}

impl ToString for Use
{
	fn to_string(&self) -> String
	{
		let ref me = *self;
		match me
		{
			&Use::Static => return "static".to_string(),
			&Use::Dynamic => return "dynamic".to_string(),
		}
	}
}

impl Recipe
{
	pub fn new() -> Recipe
	{
		Recipe
		{
			ok: true,
			path: PathBuf::new(),
			target_count: 0,
			targets: Vec::new(),
		}
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

	//TODO
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
			if line.starts_with("#") { continue; }
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

							//check for extra tokens
							match tokens.next()
							{
								Some(s) =>
								{
									if errors
										{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
									return;
								},
								None => {},
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
							match tokens.next()
							{
								Some(s) =>
								{
									if errors
										{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
									return;
								},
								None => {},
							}
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
						"end" =>
						{ 
							self.targets.push(target.clone());
							self.target_count += 1;
							target = Target::new();
							state = ReadState::Start; 

							//check for extra tokens
							match tokens.next()
							{
								Some(s) =>
								{
									if errors
										{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
									return;
								},
								None => {},
							}
						},
						"$refs" =>
						{
							target.options.refs = true;

							//check for extra tokens
							match tokens.next()
							{
								Some(s) =>
								{
									if errors
										{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
									return;
								},
								None => {},
							}
						}
						"$deps" =>
						{
							target.options.deps = true;
						
							//check for extra tokens
							match tokens.next()
							{
								Some(s) =>
								{
									if errors
										{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
									return;
								},
								None => {},
							}
						}
						"$nolibc" =>
						{
							target.options.nolibc = true;

							//check for extra tokens
							match tokens.next()
							{
								Some(s) =>
								{
									if errors
										{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
									return;
								},
								None => {},
							}
						}
						"$generate_ir" => 
						{ 
							target.options.generate_ir = true;

							//check for extra tokens
							match tokens.next()
							{
								Some(s) =>
								{
									if errors
										{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
									return;
								},
								None => {},
							}
						}
						"$generate_c" =>
						{
							target.options.generate_c = true;
							
							//check for extra tokens
							match tokens.next()
							{
								Some(s) =>
								{
									if errors
										{println!("error: unexpected token '{}' at line '{}'", s, line_number);}
									return;
								},
								None => {},
							}
						}
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
					},
					None => {}
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

		//TODO
		let _ = writeln!(f, "# this file is generated by recipe-reader, it might be overwritten\n");
		for trg_ptr in &self.targets
		{
			let ref trg = *trg_ptr;
			match trg.kind
			{
				TargetType::Executable => {let _ = writeln!(f, "executable {}", trg.name);},
				TargetType::SharedLib | TargetType::StaticLib => {let _ = writeln!(f, "lib {} {}", trg.name, trg.kind.to_string());},
				TargetType::Temporary => panic!("error: temporary target type is invalid"),
			}

			if trg.options.generate_c 	{let _ = writeln!(f, "\t$generate-c");}
			if trg.options.generate_ir 	{let _ = writeln!(f, "\t$generate-ir");}
			if trg.options.nolibc 		{let _ = writeln!(f, "\t$nolibc");}
			if trg.options.deps 		{let _ = writeln!(f, "\t$deps");}
			if trg.options.refs 		{let _ = writeln!(f, "\t$refs");}
			if trg.options.export.len() > 0
			{
				let _ = write!(f, "\t$export");
				for export in &trg.options.export
				{
					let _ = write!(f, " {}", export);
				}
				let _ = write!(f, "\n");
			}
			if trg.options.config.len() > 0
			{
				let _ = write!(f, "\t$config");
				for config in &trg.options.config
				{
					let _ = write!(f, " {}", config);
				}
				let _ = write!(f, "\n");
			}
			if trg.options.warnings.len() > 0
			{
				let _ = write!(f, "\t$warnings");
				for warning in &trg.options.warnings
				{
					let _ = write!(f, " {}", warning);
				}
				let _ = write!(f, "\n");
			}
			if trg.options.lib_use.len() > 0
			{
				for lib_use in &trg.options.lib_use
				{
					let _ = writeln!(f, "\t$use {} {}", lib_use.0, lib_use.1.to_string());
				}
			}
			for file in &trg.files
			{
				let _ = writeln!(f, "\t{}", file);
			}
			let _ = writeln!(f, "end\n");
		}
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
