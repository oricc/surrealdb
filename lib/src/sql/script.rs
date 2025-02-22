use crate::sql::error::IResult;
use nom::branch::alt;
use nom::bytes::complete::escaped;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::one_of;
use nom::combinator::recognize;
use nom::multi::many1;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str;

const SINGLE: &str = r#"'"#;
const SINGLE_ESC: &str = r#"\'"#;

const DOUBLE: &str = r#"""#;
const DOUBLE_ESC: &str = r#"\""#;

const BACKTICK: &str = r#"`"#;
const BACKTICK_ESC: &str = r#"\`"#;

const OBJECT_BEG: &str = "{";
const OBJECT_END: &str = "}";

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Script(pub String);

impl From<String> for Script {
	fn from(s: String) -> Self {
		Self(s)
	}
}

impl From<&str> for Script {
	fn from(s: &str) -> Self {
		Self::from(String::from(s))
	}
}

impl Deref for Script {
	type Target = String;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Display for Script {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

pub fn script(i: &str) -> IResult<&str, Script> {
	let (i, v) = recognize(script_raw)(i)?;
	Ok((i, Script(String::from(v))))
}

fn script_raw(i: &str) -> IResult<&str, &str> {
	recognize(many1(alt((char_any, char_object, string_single, string_double, string_backtick))))(i)
}

fn char_any(i: &str) -> IResult<&str, &str> {
	is_not("{}'`\"")(i)
}

fn char_object(i: &str) -> IResult<&str, &str> {
	let (i, _) = tag(OBJECT_BEG)(i)?;
	let (i, v) = script_raw(i)?;
	let (i, _) = tag(OBJECT_END)(i)?;
	Ok((i, v))
}

fn string_single(i: &str) -> IResult<&str, &str> {
	let (i, _) = tag(SINGLE)(i)?;
	let (i, v) = alt((escaped(is_not(SINGLE_ESC), '\\', one_of(SINGLE)), tag("")))(i)?;
	let (i, _) = tag(SINGLE)(i)?;
	Ok((i, v))
}

fn string_double(i: &str) -> IResult<&str, &str> {
	let (i, _) = tag(DOUBLE)(i)?;
	let (i, v) = alt((escaped(is_not(DOUBLE_ESC), '\\', one_of(DOUBLE)), tag("")))(i)?;
	let (i, _) = tag(DOUBLE)(i)?;
	Ok((i, v))
}

fn string_backtick(i: &str) -> IResult<&str, &str> {
	let (i, _) = tag(BACKTICK)(i)?;
	let (i, v) = alt((escaped(is_not(BACKTICK_ESC), '\\', one_of(BACKTICK)), tag("")))(i)?;
	let (i, _) = tag(BACKTICK)(i)?;
	Ok((i, v))
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn script_basic() {
		let sql = "return true;";
		let res = script(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("return true;", format!("{}", out));
		assert_eq!(out, Script::from("return true;"));
	}

	#[test]
	fn script_object() {
		let sql = "return { test: true, something: { other: true } };";
		let res = script(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("return { test: true, something: { other: true } };", format!("{}", out));
		assert_eq!(out, Script::from("return { test: true, something: { other: true } };"));
	}

	#[test]
	fn script_closure() {
		let sql = "return this.values.map(v => `This value is ${Number(v * 3)}`);";
		let res = script(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!(
			"return this.values.map(v => `This value is ${Number(v * 3)}`);",
			format!("{}", out)
		);
		assert_eq!(
			out,
			Script::from("return this.values.map(v => `This value is ${Number(v * 3)}`);")
		);
	}

	#[test]
	fn script_complex() {
		let sql = r#"return { test: true, some: { object: "some text with uneven {{{ {} \" brackets", else: false } };"#;
		let res = script(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!(
			r#"return { test: true, some: { object: "some text with uneven {{{ {} \" brackets", else: false } };"#,
			format!("{}", out)
		);
		assert_eq!(
			out,
			Script::from(
				r#"return { test: true, some: { object: "some text with uneven {{{ {} \" brackets", else: false } };"#
			)
		);
	}
}
