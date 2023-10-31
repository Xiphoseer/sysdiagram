use encoding::{all::UTF_16LE, DecoderTrap, Encoding};
use ms_oforms::properties::parse_size;
use nom::bytes::complete::{tag, take, take_until};
use nom::combinator::{map_res, recognize, verify};
use nom::error::{FromExternalError, ParseError};
use nom::multi::{count, length_value, many_till};
use nom::number::complete::{le_u16, le_u32};
use nom::sequence::pair;
use nom::IResult;
use std::borrow::Cow;
use std::convert::TryFrom;

use crate::{SchGrid, SchGridInner};

fn parse_wstring_nt(input: &[u8]) -> IResult<&[u8], String> {
    map_res(
        recognize(many_till(le_u16, tag([0x00, 0x00]))),
        |x: &[u8]| UTF_16LE.decode(&x[..(x.len() - 2)], DecoderTrap::Strict),
    )(input)
}

fn decode_utf16(input: &[u8]) -> Result<String, Cow<'static, str>> {
    UTF_16LE.decode(input, DecoderTrap::Strict)
}

fn le_u32_2(input: &[u8]) -> IResult<&[u8], (u32, u32)> {
    pair(le_u32, le_u32)(input)
}

pub(crate) fn parse_u32_bytes_wstring_nt<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], String, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, len) = le_u32(input)?;
    let (input, string) = map_res(take(len - 2), decode_utf16)(input)?;
    let (input, _) = tag([0x00, 0x00])(input)?;
    Ok((input, string))
}

pub(crate) fn parse_u32_wstring_nt(input: &[u8]) -> IResult<&[u8], String> {
    let (input, len) = le_u32(input)?;
    let (input, string) = map_res(take(len * 2 - 2), decode_utf16)(input)?;
    let (input, _) = tag([0x00, 0x00])(input)?;
    Ok((input, string))
}

pub(crate) fn parse_u16_wstring<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], String, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, len) = le_u16(input)?;
    map_res(take((len as usize) << 1), decode_utf16)(input)
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

fn parse_sch_grid_inner(input: &[u8]) -> IResult<&[u8], SchGridInner> {
    let (input, v) = count(le_u32, 11)(input)?;
    Ok((input, SchGridInner(v)))
}

// See: <https://github.com/jandubois/win32-ole/blob/27570c90dcb3cf56ef815f668cc346dc0ac099a3/OLE.xs#L151>
const WINOLE_MAGIC: u32 = 0x1234_4321;

pub fn parse_sch_grid(input: &[u8]) -> IResult<&[u8], SchGrid> {
    let (input, d1) = verify(le_u32, |x| *x == WINOLE_MAGIC)(input)?;
    let (input, d2) = le_u32(input)?;
    let (input, size1) = parse_size(input)?;
    let (input, d3) = verify(le_u32, |x| *x == 0x1234_5678)(input)?;
    let (input, d4) = le_u32(input)?;
    let (input, name) = length_value(le_u32, parse_wstring_nt)(input)?;
    let (input, d5_1) = le_u32_2(input)?;
    let (input, d5_2) = le_u32_2(input)?;
    let (input, d5_3) = le_u32_2(input)?;
    let (input, d6) = le_u32(input)?;
    let (input, _d7) = count(le_u32, 16usize)(input)?;
    let (input, size2) = parse_size(input)?;
    let (input, d8_0) = le_u32(input)?;
    let (input, col_count) = le_u32(input)?;
    let (input, cols_shown) = le_u32(input)?;
    let (input, x1) = count(parse_sch_grid_inner, 4)(input)?;
    //let (input, _d8) = count(le_u32, 13usize)(input)?;
    //let (input, d9) = le_u32(input)?;
    //let (input, _d10) = count(le_u32, 8usize)(input)?;
    //let (input, _d11) = take(8usize * 4)(input)?;
    //let (input, d12) = le_u32(input)?;
    //let (input, d13) = le_u32_2(input)?;
    let (input, d14_len) = map_res(le_u32, usize::try_from)(input)?;
    let (input, d14) = count(le_u32, d14_len)(input)?;
    let (input, schema) = parse_u32_wstring_nt(input)?;
    let (input, table) = parse_u32_wstring_nt(input)?;
    Ok((
        input,
        SchGrid {
            d1,
            d2,
            size1,
            d3,
            d4,
            name,
            d5_1,
            d5_2,
            d5_3,
            d6,
            d7: _d7,
            size2,
            d8_0,
            col_count,
            cols_shown,
            x1,
            //d8: _d8,
            //d9,
            //d10: _d10,
            //d11: BString::from(_d11),
            //d12,
            //d13,
            d14,
            schema,
            table,
        },
    ))
}
