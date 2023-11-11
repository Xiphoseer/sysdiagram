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

use bitflags::bitflags;
use bstr::BString;
use ms_oforms::properties::{
    color::{parse_ole_color, OleColor},
    font::{parse_std_font, StdFont},
    Position, Size,
};
use nom::{
    bytes::complete::take,
    combinator::{map, map_opt, rest},
    error::{FromExternalError, ParseError},
    multi::{count, length_count},
    number::complete::{le_u16, le_u32, le_u8},
    IResult,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::borrow::Cow;
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
/// Microsoft DDS Form 2.0 (aka `MSDDS.Form.080.1`)
pub const CLSID_DDS_FORM: Uuid = uuid!("77d2c92e-7779-11d8-9070-00065b840d9c");

/// Microsoft DT DDSForm (`mdt2df.dll`)
pub const TYPELIB_DDS_FORM: Uuid = uuid!("105b80d0-95f1-11d0-b0a0-00aa00bdcb5c");

/// Microsoft DT DDSform 2.1 FormPackage
pub const CLSID_DDS2_FORM_PACKAGE: Uuid = uuid!("105b80d5-95f1-11d0-b0a0-00aa00bdcb5c");

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub enum DdsPolylineEndType {
    Many = 0,
    LittleNub = 1,
    Key = 2,
    SingleArrowFill = 3,
    DoubleArrow = 4,
    RoundNub = 5,
    None = 6,
    OpenArrow = 7,
    SingleArrow = 8,
    Diamond = 9,
    DiamondFill = 10,
    DiamondArrow = 11,
    DiamondFillArrow = 12,
    ManyDelete = 13,
    ManyUpdate = 14,
    ManyUpdateDelete = 15,
    KeyDelete = 16,
    KeyUpdate = 17,
    KeyUpdateDelete = 18,
    Custom = 99,
}

#[derive(Debug)]
pub struct LabelRef {
    pub id: u32,
    pub(crate) _x2: u32, // 0
    pub pos: Position,
    pub size: Size,
}

/// ## Polyline
///
/// See also: <https://wutils.com/com-dll/constants/constants-MSDDS.htm>
#[derive(Debug)]
pub struct Polyline {
    pub(crate) _d1: u16, // 11 ? dpetDiamondArrow ?
    pub positions: Vec<Position>,
    pub end_type_src: DdsPolylineEndType,  // 0 (dpetMany ?)
    pub end_type_dest: DdsPolylineEndType, // 2 (dlotConnector ?, dbvUIActiveVisible ? dpcetsRect ? dpcetcsLineColor ? dpetKey ?)
    pub color: OleColor,
    pub(crate) _x1: BString, // (16) GUID NIL?, Color Black?
    pub labels: Vec<LabelRef>,
    pub(crate) _d7: u8,        // 0b0011_1111 flags ??
    pub(crate) _rest: BString, // "\0\0\0\x01\0"
}

#[derive(Debug)]
pub struct Label {
    pub(crate) _d1: u32, // 0x02 = label pos type?
    pub size: Size,
    pub(crate) _d2: BString, // 0x02 = label pos type?
    pub back_color: OleColor,
    pub fore_color: OleColor,
    pub justification: LabelJustification,
    pub(crate) _d3: u16,
    pub flags: LabelFlags,
    pub font: StdFont,
    pub text: String,
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct LabelFlags: u16 {
        const READ_ONLY = 0b000001;
        const ALIGN_TOP = 0b000010; // vertical center = off
        const AUTO_SIZE = 0b000100;
        const DELETE_EMPTY = 0b001000;
        const WORD_WRAP = 0b010000;
        const TRANSPARENT = 0b100000;
    }
}

/// Horizontal Justification on the label
///
/// See: <https://wutils.com/com-dll/constants/constants-DDSLibrary.htm>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelJustification {
    Left = 0,
    Center = 1,
    Right = 2,
}

impl LabelJustification {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            0 => Some(LabelJustification::Left),
            1 => Some(LabelJustification::Center),
            2 => Some(LabelJustification::Right),
            _ => None,
        }
    }
}

pub fn parse_label<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Label, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
    E: FromExternalError<&'a [u8], u32>,
{
    let (input, _d1) = le_u32(input)?;
    let (input, size) = Size::parse(input)?;

    let (input, _d2) = map(take(6usize), BString::from)(input)?;
    let (input, back_color) = parse_ole_color(input)?;
    let (input, fore_color) = parse_ole_color(input)?;
    let (input, justification) = map_opt(le_u16, LabelJustification::from_u16)(input)?;
    let (input, _d3) = le_u16(input)?;
    let (input, flags) = map_opt(le_u16, LabelFlags::from_bits)(input)?;
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
            justification,
            _d3,
            flags,
            font,
            text,
        },
    ))
}

fn parse_label_ref(input: &[u8]) -> IResult<&[u8], LabelRef> {
    let (input, id) = le_u32(input)?;
    let (input, _x2) = le_u32(input)?;
    let (input, pos) = Position::parse(input)?;
    let (input, size) = Size::parse(input)?;
    Ok((input, LabelRef { id, _x2, pos, size }))
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
    let (input, _d1) = le_u16(input)?;
    let (input, positions) = count(Position::parse, usize::from(pos_count))(input)?;
    let (input, end_type_src) = map_opt(le_u32, DdsPolylineEndType::from_u32)(input)?;
    let (input, end_type_dest) = map_opt(le_u32, DdsPolylineEndType::from_u32)(input)?;
    let (input, color) = parse_ole_color(input)?;
    let (input, _x1) = map(take(16usize), BString::from)(input)?;
    let (input, labels) = length_count(le_u32, parse_label_ref)(input)?;
    let (input, _d7) = le_u8(input)?;
    /*let (input, _d8) = take(6usize)(input)?;
    let (input, d9) = le_u32(input)?;*/
    let (input, _rest) = map(rest, BString::from)(input)?;
    Ok((
        input,
        Polyline {
            _d1,
            positions,
            end_type_src,
            end_type_dest,
            color,
            _x1,
            labels,
            _d7,
            /*d8, d9,*/
            _rest,
        },
    ))
}
