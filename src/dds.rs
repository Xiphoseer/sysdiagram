//! # DaVinci Design Surface (DDS)
//!
//! From: <https://social.msdn.microsoft.com/Forums/en-US/1d88df38-23d4-48a6-ab77-3b99dc1e9a24/why-binary-annotations?forum=sqlintegrationservices>
//! > DDS (DaVinci Design Surface) is an internal name for the design surface we used for our designers.
//! > It is originally developed by VS and I think it was also used for a few other designers too
//! > (like the UI designers in VS 6). It is a pretty old design surface and it will soon be replaced everywhere it was used.
//!
//! See also:
//! - <https://fjehl.wordpress.com/2010/05/23/ssis-modifier-lapparence-des-packages-via-lapi-fail/>
//! - <http://sqlsoundings.blogspot.com/2011/10/ssis-word-wrapping-annotations-using.html>
//! - <https://www.sqlservercentral.com/articles/hidden-ssis-features-word-wrapping-your-annotations-and-more>

use std::borrow::Cow;

use bstr::BString;
use ms_oforms::properties::{
    color::{parse_ole_color, OleColor},
    font::{parse_std_font, StdFont},
    parse_position, parse_size, Position, Size,
};
use nom::{
    bytes::complete::take,
    combinator::{map, rest},
    error::{FromExternalError, ParseError},
    multi::count,
    number::complete::{le_u16, le_u32},
    IResult,
};
use uuid::{uuid, Uuid};

use crate::parse_u16_wstring;

/// Microsoft DT PolyLine Control 2 (ProgID `MSDTPolylineControl.2`)
pub const CLSID_POLYLINE: Uuid = uuid!("d24d4453-1f01-11d1-8e63-006097d2df48");
/// Microsoft DT Label Control (ProgID `MSDTDDSLabel.1`)
pub const CLSID_DDSLABEL: Uuid = uuid!("d24d4451-1f01-11d1-8e63-006097d2df48");
/// Microsoft DT Diagram Surface 2 (ProgID `MSDTDDS.2`)
pub const CLSID_MSDTDDS: Uuid = uuid!("b0406340-b0c5-11d0-89a9-00a0c9054129");
/// Microsoft DT DDS TypeLib 2 (`mdt2dd.dll`)
pub const TYPELIB_DTDDS2: Uuid = uuid!("b0406341-b0c5-11d0-89a9-00a0c9054129");
/// Microsoft DT Typelib
pub const TYPELIB_MSDT: Uuid = uuid!("a92cc3f0-e7c4-11ce-a47b-00aa005119fb");

/// Microsoft DT DDS Form 2.0 (aka `MDTDF.Form.1`)
pub const CLSID_MSDT_DDS_FORM_2: Uuid = uuid!("105b80d2-95f1-11d0-b0a0-00aa00bdcb5c");

/// Microsoft DT DDSForm (`mdt2df.dll`)
pub const TYPELIB_DDS_FORM: Uuid = uuid!("105b80d0-95f1-11d0-b0a0-00aa00bdcb5c");

/// Microsoft DT DDSform 2.1 FormPackage
pub const CLSID_DDS2_FORM_PACKAGE: Uuid = uuid!("105b80d5-95f1-11d0-b0a0-00aa00bdcb5c");

#[derive(Debug)]
pub struct Polyline {
    pub d1: u16,
    pub positions: Vec<Position>,
    //pub pos: Position,
    //pub d2: [u8; 32],
    //pub d3: u32,
    //pub d4: u32,
    //pub d5: u32,
    //pub d6: u32,
    //pub d7: u32,
    //pub d8: [u8; 6],
    //pub d9: u32,
    pub(crate) _rest: BString,
}

#[derive(Debug)]
pub struct Label {
    pub(crate) _d1: u32,
    pub size: Size,
    pub(crate) _d2: BString,
    pub back_color: OleColor,
    pub fore_color: OleColor,
    pub(crate) _d3: u32,
    pub(crate) _flags: u16,
    pub font: StdFont,
    pub text: String,
}

pub fn parse_label<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Label, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
    E: FromExternalError<&'a [u8], u32>,
{
    let (input, _d1) = le_u32(input)?;
    let (input, size) = parse_size(input)?;

    let (input, _d2) = map(take(6usize), BString::from)(input)?;
    let (input, back_color) = parse_ole_color(input)?;
    let (input, fore_color) = parse_ole_color(input)?;
    let (input, _d3) = le_u32(input)?;
    let (input, _flags) = le_u16(input)?;
    let (input, font) = parse_std_font(input)?;
    let (input, text) = parse_u16_wstring(input)?;
    Ok((
        input,
        Label {
            _d1,
            size,
            _d2,
            back_color,
            fore_color,
            _d3,
            _flags,
            font,
            text,
        },
    ))
}

// See:
// - <https://wutils.com/com-dll/constants/constants-MSDDS.htm>
// - <https://wutils.com/com-dll/constants/constants-MSDDSForm.htm>
// - <https://wutils.com/com-dll/constants/constants-MSDDSLM.htm>
// - <https://wutils.com/com-dll/constants/constants-DDSLibrary.htm>
// - <https://wutils.com/com-dll/constants/constants-DdsShapesLib.htm>
// - <https://wutils.com/com-dll/constants/constants-DEDesignerExtensibility.htm>
// - <https://wutils.com/com-dll/constants/constants-VBDataView.htm>
// - <https://wutils.com/com-dll/constants/constants-VBDataViewSupport.htm>
pub fn parse_polyline(input: &[u8]) -> IResult<&[u8], Polyline> {
    let (input, pos_count) = le_u16(input)?;
    let (input, d1) = le_u16(input)?;
    let (input, positions) = count(parse_position, usize::from(pos_count))(input)?;
    /*let (input, _d2) = take(32usize)(input)?;
    let (input, d3) = le_u32(input)?;
    let (input, d4) = le_u32(input)?;
    let (input, pos) = parse_position(input)?;
    let (input, d5) = le_u32(input)?;
    let (input, d6) = le_u32(input)?;
    let (input, d7) = le_u32(input)?;
    let (input, _d8) = take(6usize)(input)?;
    let (input, d9) = le_u32(input)?;*/
    let (input, _rest) = map(rest, BString::from)(input)?;
    Ok((
        input,
        Polyline {
            //pos,
            d1,
            positions,
            /*d2, d3,
            d4,
            d5,
            d6,
            d7,
            d8, d9,*/
            _rest,
        },
    ))
}
