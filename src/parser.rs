use encoding_rs::UTF_16LE;
use nom::bytes::complete::{tag, take, take_until};
use nom::combinator::{map, map_opt, recognize};
use nom::error::{FromExternalError, ParseError};
use nom::multi::many_till;
use nom::number::complete::{le_u16, le_u32};
use nom::sequence::pair;
use nom::IResult;
use std::borrow::Cow;

fn decode_utf16(input: &[u8]) -> Option<String> {
    UTF_16LE
        .decode_without_bom_handling_and_without_replacement(input)
        .map(Cow::into_owned)
}

pub(crate) fn parse_wstring_nt(input: &[u8]) -> IResult<&[u8], String> {
    map_opt(
        map(
            recognize(many_till(le_u16, tag([0x00, 0x00]))),
            |x: &[u8]| &x[..(x.len() - 2)],
        ),
        decode_utf16,
    )(input)
}

pub(crate) fn le_u32_2(input: &[u8]) -> IResult<&[u8], (u32, u32)> {
    pair(le_u32, le_u32)(input)
}

pub(crate) fn parse_u32_bytes_wstring_nt<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], String, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, len) = le_u32(input)?;
    let (input, string) = map_opt(take(len - 2), decode_utf16)(input)?;
    let (input, _) = tag([0x00, 0x00])(input)?;
    Ok((input, string))
}

pub(crate) fn parse_u32_wstring_nt(input: &[u8]) -> IResult<&[u8], String> {
    let (input, len) = le_u32(input)?;
    let (input, string) = map_opt(take(len * 2 - 2), decode_utf16)(input)?;
    let (input, _) = tag([0x00, 0x00])(input)?;
    Ok((input, string))
}

pub(crate) fn parse_u16_wstring<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], String, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, len) = le_u16(input)?;
    map_opt(take((len as usize) << 1), decode_utf16)(input)
}

pub fn parse_relationship(input: &str) -> IResult<&str, (String, String, String)> {
    let (input, _) = tag("Relationship '")(input)?;
    let (input, name) = take_until("'")(input)?;
    let (input, _) = tag("' between '")(input)?;
    let (input, from) = take_until("'")(input)?;
    let (input, _) = tag("' and '")(input)?;
    let (input, to) = take_until("'")(input)?;
    Ok((input, (name.to_string(), from.to_string(), to.to_string())))
}
