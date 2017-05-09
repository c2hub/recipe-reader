/*
** Tests - test with 'cargo run -- --no-capture --test-threads 1'
** -> all tests use the same recipe file, which is being replaced each time,
** which means tests cause race conditions when run on multiple threads
*/

use std::fs::{File, remove_file};
use std::path::Path;
use super::*;

macro_rules! recipe_prep
{
	($name:expr) =>
	({
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

#[test]
fn library_unknown_type()
{
	recipe_prep!("tests/library_unknown_type");
	not_ok_test!();
}

#[test]
fn unknown_option()
{
	recipe_prep!("tests/unknown_option");
	not_ok_test!();
}

#[test]
fn extra_nolibc_token()
{
	recipe_prep!("tests/extra_nolibc_token");
	not_ok_test!();
}

#[test]
fn use_option_missing_args()
{
	recipe_prep!("tests/use_option_missing_args");
	not_ok_test!();
}

#[test]
fn missing_target_name()
{
	recipe_prep!("tests/missing_target_name");
	not_ok_test!();
}

#[test]
fn extra_deps_token()
{
	recipe_prep!("tests/extra_deps_token");
	not_ok_test!();
}

#[test]
fn extra_end_token()
{
	recipe_prep!("tests/extra_end_token");
	not_ok_test!();
}

#[test]
fn extra_refs_token()
{
	recipe_prep!("tests/extra_refs_token");
	not_ok_test!();
}

#[test]
fn extra_generate_ir_token()
{
	recipe_prep!("tests/extra_generate_ir_token");
	not_ok_test!();
}

#[test]
fn extra_generate_c_token()
{
	recipe_prep!("tests/extra_generate_c_token");
	not_ok_test!();
}

#[test]
fn many_empty_lines()
{
	recipe_prep!("tests/many_empty_lines");
	ok_test!();
}

#[test]
fn config_more_than_one()
{
	recipe_prep!("tests/config_more_than_one");
	ok_test!();
}

#[test]
fn export_more_than_one()
{
	recipe_prep!("tests/export_more_than_one");
	ok_test!();
}
