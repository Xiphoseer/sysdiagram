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
use nom::combinator::map;
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
    pub frame: Box<GridFrameWnd>,
    pub data_source: DataSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridFrameWnd {
    pub caption: String,
    pub x1: GridSpec,
    /// - .count Probably the number of columns on this table
    /// - .shown is min(.count, 12)
    /// - .size is identical to the main view extent
    pub cols: GridSpec,
    pub keys: GridSpec,
    /// - .count/.shown is possibly the number of key constraints on this table (primary + foreign)
    pub x2: GridSpec,
    pub x3: GridSpec,
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
pub struct GridSpec {
    pub v0: (u32, u32),
    pub v1: Size,
    pub v2: u32,
    pub count: u32,
    pub shown: u32,
    pub v5: u32,
    pub v6: Vec<u32>,
}

fn parse_grid_spec(input: &[u8]) -> IResult<&[u8], GridSpec> {
    let (input, v0) = le_u32_2(input)?;
    let (input, v1) = Size::parse(input)?;
    let (input, v2) = le_u32(input)?;
    let (input, _count) = le_u32(input)?;
    let (input, shown) = le_u32(input)?;
    let (input, (v6_count, v5)) = pair(le_u32, le_u32)(input)?;
    let (input, v6) = count(le_u32, v6_count as usize)(input)?;
    Ok((
        input,
        GridSpec {
            v0,
            v1,
            v2,
            count: _count,
            shown,
            v5,
            v6,
        },
    ))
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

fn parse_grid_frame_wnd(input: &[u8]) -> IResult<&[u8], GridFrameWnd> {
    let (input, _) = tag(u32::to_le_bytes(0x1234_5678))(input)?;
    let (input, (v_minor, v_major)) = pair(le_u16, le_u16)(input)?;
    assert_eq!((v_minor, v_major), (7, 0));
    let (input, name) = length_value(le_u32, parse_wstring_nt)(input)?;
    let (input, x1) = parse_grid_spec(input)?;
    let (input, cols) = parse_grid_spec(input)?;
    let (input, keys) = parse_grid_spec(input)?;
    let (input, x2) = parse_grid_spec(input)?;
    let (input, x3) = parse_grid_spec(input)?;
    Ok((
        input,
        GridFrameWnd {
            caption: name,
            x1,
            cols,
            keys,
            x2,
            x3,
        },
    ))
}

pub fn parse_sch_grid(input: &[u8]) -> IResult<&[u8], SchGrid> {
    let (input, extent) = parse_ole_control_extent(input)?;
    let (input, frame) = map(parse_grid_frame_wnd, Box::new)(input)?;
    let (input, data_source) = parse_data_source(input)?;

    Ok((
        input,
        SchGrid {
            extent,
            frame,
            data_source,
        },
    ))
}
