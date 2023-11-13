use std::borrow::Cow;

use ms_oforms::common::VarType;
use nom::{
    combinator::{map, map_opt},
    error::{FromExternalError, ParseError},
    number::complete::le_u16,
    IResult,
};

use crate::parse_u32_bytes_wstring_nt;

#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Variant {
    BStr(String),
    Bool(bool),
}

pub fn parse_variant<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Variant, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, vt) = map_opt(le_u16, VarType::from_bits)(input)?;
    let (input, value) = match vt {
        VarType::BSTR => map(parse_u32_bytes_wstring_nt, Variant::BStr)(input),
        VarType::BOOL => map(
            map_opt(le_u16, |v| match v {
                0x0000 => Some(false),
                0xFFFF => Some(true),
                _ => None,
            }),
            Variant::Bool,
        )(input),
        _ => todo!("0x{:04x}", vt),
    }?;
    Ok((input, value))
}
