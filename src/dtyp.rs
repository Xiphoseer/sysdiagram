use std::borrow::Cow;

use ms_oforms::common::VarType;
use nom::{
    combinator::{map, map_opt},
    error::{FromExternalError, ParseError},
    number::complete::le_u16,
    IResult,
};

use crate::parse_u32_bytes_wstring_nt;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Variant {
    BStr(String),
}

pub fn parse_variant<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Variant, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, vt) = map_opt(le_u16, VarType::from_bits)(input)?;
    let (input, value) = match vt {
        VarType::BSTR => map(parse_u32_bytes_wstring_nt, Variant::BStr)(input),
        _ => todo!("0x{:04x}", vt),
    }?;
    Ok((input, value))
}
