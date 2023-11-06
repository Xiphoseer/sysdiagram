use encoding_rs::UTF_16LE;
use ms_oforms::properties::parse_size;
use nom::bytes::complete::{tag, take, take_until};
use nom::combinator::{map, map_opt, map_res, recognize};
use nom::error::{FromExternalError, ParseError};
use nom::multi::{count, length_value, many_till};
use nom::number::complete::{le_u16, le_u32};
use nom::sequence::{pair, tuple};
use nom::IResult;
use std::borrow::Cow;
use std::convert::TryFrom;

use crate::{OleControlExtent, SchGrid, SchGridB, SchGridC, SchGridInner};

fn decode_utf16(input: &[u8]) -> Option<String> {
    UTF_16LE
        .decode_without_bom_handling_and_without_replacement(input)
        .map(Cow::into_owned)
}

fn parse_wstring_nt(input: &[u8]) -> IResult<&[u8], String> {
    map_opt(
        map(
            recognize(many_till(le_u16, tag([0x00, 0x00]))),
            |x: &[u8]| &x[..(x.len() - 2)],
        ),
        decode_utf16,
    )(input)
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

fn parse_sch_grid_inner(input: &[u8]) -> IResult<&[u8], SchGridInner> {
    let (input, v1) = count(le_u32, 6)(input)?;
    let (input, size) = parse_size(input)?;
    let (input, v2) = count(le_u32, 3)(input)?;
    Ok((input, SchGridInner(v1, size, v2)))
}

// See:
// - <https://github.com/jandubois/win32-ole/blob/27570c90dcb3cf56ef815f668cc346dc0ac099a3/OLE.xs#L151>
// - <https://github.com/LibreOffice/core/blob/b4e7ebebd583a2a3856231aead66d72d3bc1cb46/oox/source/ole/axcontrol.cxx#L722>
const OLE_CONTROL_MAGIC: u32 = 0x1234_4321;

// See: <https://github.com/LibreOffice/core/blob/b4e7ebebd583a2a3856231aead66d72d3bc1cb46/oox/source/ole/axcontrol.cxx#L720-L729>
fn parse_ole_control_extent(input: &[u8]) -> IResult<&[u8], OleControlExtent> {
    let (input, _) = tag(OLE_CONTROL_MAGIC.to_le_bytes())(input)?;
    let (input, _) = tuple((
        tag(u16::to_le_bytes(8)), // minor
        tag(u16::to_le_bytes(0)), // major
    ))(input)?;
    let (input, size) = parse_size(input)?;
    Ok((input, OleControlExtent { size }))
}

pub fn parse_sch_grid(input: &[u8]) -> IResult<&[u8], SchGrid> {
    let (input, extent) = parse_ole_control_extent(input)?;
    let (input, _) = tag(u32::to_le_bytes(0x1234_5678))(input)?;
    let (input, d4) = le_u32(input)?;
    let (input, name) = length_value(le_u32, parse_wstring_nt)(input)?;
    let (input, d5_1) = le_u32_2(input)?;
    let (input, _d5_2) = parse_size(input)?;
    let (input, d5_3) = le_u32_2(input)?;
    let (input, d6) = le_u32(input)?;
    let (input, _d7) = count(le_u32, 16usize)(input)?;
    let (input, size2) = parse_size(input)?;
    let (input, d8_0) = le_u32(input)?;
    let (input, col_count) = le_u32(input)?;
    let (input, cols_shown) = le_u32(input)?;
    let (input, x1) = count(parse_sch_grid_inner, 3)(input)?;
    let (input, x2) = count(le_u32, 6usize)(input)?;
    //let (input, _d8) = count(le_u32, 13usize)(input)?;
    //let (input, d9) = le_u32(input)?;
    //let (input, _d10) = count(le_u32, 8usize)(input)?;
    //let (input, _d11) = take(8usize * 4)(input)?;
    //let (input, d12) = le_u32(input)?;
    //let (input, d13) = le_u32_2(input)?;
    let (input, _) = tag(u32::to_le_bytes(0x1234_5678))(input)?;
    let (input, _cd1) = le_u32(input)?;
    let (input, _cd2) = le_u32(input)?;
    let (input, _cd3) = le_u32(input)?;
    let (input, _cd4) = le_u32(input)?;

    let (input, d14_len) = map_res(le_u32, usize::try_from)(input)?;
    let (input, d14) = count(le_u32, d14_len)(input)?;
    let (input, schema) = parse_u32_wstring_nt(input)?;
    let (input, table) = parse_u32_wstring_nt(input)?;
    Ok((
        input,
        SchGrid {
            extent,
            b: SchGridB {
                _d4: d4,
                name,
                _d5_1: d5_1,
                _d5_2,
                _d5_3: d5_3,
                _d6: d6,
                _d7,
                _size2: size2,
                _d8_0: d8_0,
                col_count,
                cols_shown,
                _x1: x1,
                _x2: x2,
            },
            c: SchGridC {
                _cd1,
                _cd2,
                _cd3,
                _cd4,
                _d14: d14,
                schema,
                table,
            },
        },
    ))
}
