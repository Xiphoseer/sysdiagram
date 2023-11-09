//! ## Microsoft Data Tools Database Designer
//!
//! Also known as the `SchGrid OLE Custom Control Module` (originally `Mdtsg.ocx`, v.1.00.00.2520,
//! later `mdt2db.dll`) this ActiveX control library provided the central UI element
//! of the Database Designer form, [`SchGrid`] (aka `MSDTDDGridCtrl2`).
//!
//! From the serialization and function names, it was likely an ActiveX component
//! built with Microsoft Foundation Classes (MFC), with some the embedded window frame
//! inheriting from `CFrameWnd`.
//!
//! See also: <http://www.dejadejadeja.com/detech/ocxdb/mdt2db.dll.txt.lisp>

use crate::{le_u32_2, parse_u32_wstring_nt, parse_wstring_nt};
use ms_oforms::properties::Size;
use nom::bytes::complete::tag;
use nom::multi::{count, length_count, length_value};
use nom::number::complete::{le_u16, le_u32};
use nom::sequence::pair;
use nom::IResult;
use uuid::{uuid, Uuid};

/// `SchGrid OLE Custom Control module` (`mdt2db.dll`)
pub const TYPELIB_SCHGRID: Uuid = uuid!("e9b0e6da-811c-11d0-ad51-00a0c90f5739");

/// Microsoft Data Tools Database Designer
pub const CLSID_MSDTDB_DESIGNER: Uuid = uuid!("e9b0e6d4-811c-11d0-ad51-00a0c90f5739");

pub const PROGID_MSDTDB_DESIGNER: &str = "MSDTDatabaseDesigner2";
/// Microsoft Data Tools Database Designer SQL Server Table Property Page
pub const CLSID_MSDTDB_SQLSERVER_TABLE_PROPERTY_PAGE: Uuid =
    uuid!("e9b0e6d5-811c-11d0-ad51-00a0c90f5739");
/// Microsoft Data Tools Database Designer SQL Server Relationship Property Page
pub const CLSID_MSDTDB_SQLSERVER_RELATIONSHIP_PROPERTY_PAGE: Uuid =
    uuid!("e9b0e6d6-811c-11d0-ad51-00a0c90f5739");
/// Microsoft Data Tools Database Designer SQL Server Index Property Page
pub const CLSID_MSDTDB_SQLSERVER_INDEX_PROPERTY_PAGE: Uuid =
    uuid!("e9b0e6d8-811c-11d0-ad51-00a0c90f5739");
/// MSDTDDGridCtrl2 Object (ProgID `SchGrid.MSDTDDGridCtrl2.1`)
pub const CLSID_SCHGRID: Uuid = uuid!("e9b0e6d9-811c-11d0-ad51-00a0c90f5739");

pub const IID_ISCHGRID_ALT: Uuid = uuid!("91a88675-8bc8-11ce-9bfd-00aa0062bebf");
pub const IID_DSCHGRID_EVENTS: Uuid = uuid!("91a88676-8bc8-11ce-9bfd-00aa0062bebf");
pub const IID_CONTROL_EVENTS: Uuid = uuid!("77d2c934-7779-11d8-9070-00065b840d9c");

pub const CLSID_ISCHGRID: Uuid = uuid!("b27d32a0-62d8-4295-8d98-273c25a2da2d");
pub const CLSID_DSCHGRID_EVENTS: Uuid = uuid!("847f3bf4-617f-43c7-8535-2986e1d552f8");

/// ## SchGrid Control
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct SchGrid {
    pub extent: Size,
    pub frame: GridFrameWnd,
    pub data_source: DataSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridFrameWnd {
    pub(crate) _d4: u32,
    pub name: String,
    pub(crate) _d5_1: (u32, u32),
    pub _d5_2: Size, // scroll container size? width bigger, height smaller (or exact, or both bigger)
    pub(crate) _d5_3: (u32, u32),
    pub(crate) _d6: u32,
    pub(crate) _d7: Vec<u32>,
    pub size: Size,
    pub(crate) _d8_0: u32,
    pub col_count: u32,
    pub cols_shown: u32, // mostly min(col_count, 12)
    //pub(crate) d8: Vec<u32>,
    //pub(crate) d9: u32,
    pub(crate) _x1: Vec<SchGridInner>,
    pub(crate) _x2: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSource {
    pub(crate) _cd3: u32,
    pub(crate) _cd4: u32,
    pub(crate) _d14: Vec<u32>, // 0 - 10, selected columns?
    pub table: String,
    pub schema: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct SchGridInner(pub(crate) Vec<u32>, pub(crate) Size, pub(crate) Vec<u32>);

fn parse_sch_grid_inner(input: &[u8]) -> IResult<&[u8], SchGridInner> {
    let (input, v1) = count(le_u32, 6)(input)?;
    let (input, size) = Size::parse(input)?;
    let (input, v2) = count(le_u32, 3)(input)?;
    Ok((input, SchGridInner(v1, size, v2)))
}

// See:
// - <https://github.com/jandubois/win32-ole/blob/27570c90dcb3cf56ef815f668cc346dc0ac099a3/OLE.xs#L151>
// - <https://github.com/LibreOffice/core/blob/b4e7ebebd583a2a3856231aead66d72d3bc1cb46/oox/source/ole/axcontrol.cxx#L722>
const OLE_CONTROL_MAGIC: u32 = 0x1234_4321;

// See: <https://github.com/LibreOffice/core/blob/b4e7ebebd583a2a3856231aead66d72d3bc1cb46/oox/source/ole/axcontrol.cxx#L720-L729>
fn parse_ole_control_extent(input: &[u8]) -> IResult<&[u8], Size> {
    let (input, _) = tag(OLE_CONTROL_MAGIC.to_le_bytes())(input)?;
    let (input, (v_minor, v_major)) = pair(le_u16, le_u16)(input)?;
    assert_eq!((v_minor, v_major), (8, 0));
    let (input, size) = Size::parse(input)?;
    Ok((input, size))
}

fn _parse_data_source(input: &[u8]) -> IResult<&[u8], DataSource> {
    let (input, _cd3) = le_u32(input)?;
    let (input, _cd4) = le_u32(input)?;
    let (input, _d14) = length_count(le_u32, le_u32)(input)?;
    let (input, schema) = parse_u32_wstring_nt(input)?;
    let (input, table) = parse_u32_wstring_nt(input)?;
    Ok((
        input,
        DataSource {
            _cd3,
            _cd4,
            _d14,
            schema,
            table,
        },
    ))
}

fn parse_data_source(input: &[u8]) -> IResult<&[u8], DataSource> {
    let (input, _) = tag(u32::to_le_bytes(0x1234_5678))(input)?;
    let (input, (v_minor, v_major)) = pair(le_u16, le_u16)(input)?;
    assert_eq!((v_minor, v_major), (4, 0));
    length_value(le_u32, _parse_data_source)(input)
}

pub fn parse_sch_grid(input: &[u8]) -> IResult<&[u8], SchGrid> {
    let (input, extent) = parse_ole_control_extent(input)?;
    let (input, _) = tag(u32::to_le_bytes(0x1234_5678))(input)?;
    let (input, d4) = le_u32(input)?;
    let (input, name) = length_value(le_u32, parse_wstring_nt)(input)?;
    let (input, d5_1) = le_u32_2(input)?;
    let (input, _d5_2) = Size::parse(input)?;
    let (input, d5_3) = le_u32_2(input)?;
    let (input, d6) = le_u32(input)?;
    let (input, _d7) = count(le_u32, 16usize)(input)?;
    let (input, size2) = Size::parse(input)?;
    let (input, d8_0) = le_u32(input)?;
    let (input, col_count) = le_u32(input)?;
    let (input, cols_shown) = le_u32(input)?;
    let (input, x1) = count(parse_sch_grid_inner, 3)(input)?;
    let (input, x2) = count(le_u32, 6usize)(input)?;
    let (input, data_source) = parse_data_source(input)?;

    Ok((
        input,
        SchGrid {
            extent,
            frame: GridFrameWnd {
                _d4: d4,
                name,
                _d5_1: d5_1,
                _d5_2,
                _d5_3: d5_3,
                _d6: d6,
                _d7,
                size: size2,
                _d8_0: d8_0,
                col_count,
                cols_shown,
                _x1: x1,
                _x2: x2,
            },
            data_source,
        },
    ))
}
