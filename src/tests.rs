/*
** Tests - test with 'cargo run -- --no-capture --test-threads 1'
*/

use std::fs::{File, remove_file};
use std::path::Path;
use super::*;

macro_rules! recipe_prep
{
	($name:expr) =>
	({
		std::sync::
		let _ = remove_file(Path::new("recipe.txt"));
		let mut f = File::open(Path::new($name)).unwrap();
		let mut recf = File::create(Path::new("recipe.txt")).unwrap();
		let mut contents: String = String::new();
		let _ = f.read_to_string(&mut contents);
		let _ = write!(recf, "{}", contents);
	});
}

macro_rules! not_ok_test
{
	() =>
	({
		let mut rec: Recipe = Recipe::new();
		rec.read_errors(true);
		assert_eq!(rec.ok, false);
		let _ = remove_file(Path::new("recipe.txt"));
	});
}

macro_rules! ok_test
{
	() =>
	({
		let mut rec: Recipe = Recipe::new();
		rec.read_errors(true);
		assert_eq!(rec.ok, true);
		let _ = remove_file(Path::new("recipe.txt"));
	});
}

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
	recipe_prep!("tests/basic_parse");

	let mut rec: Recipe = Recipe::new();
	rec.read_errors(true);
	assert_eq!(rec.ok, true);
	assert_eq!(rec.targets[0].name, "test".to_string());
	assert_eq!(rec.targets[0].kind, TargetType::Executable);
	assert_eq!(rec.targets[0].files.len(), 1);
	let _ = remove_file(Path::new("recipe.txt"));
}

#[test]
fn executable_start_extra_token()
{
	recipe_prep!("tests/executable_start_extra_token");
	not_ok_test!();
}

#[test]
fn library_start_extra_token()
{
	recipe_prep!("tests/library_start_extra_token");
	not_ok_test!();
}
