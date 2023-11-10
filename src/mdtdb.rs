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
    pub frame: Box<GridFrameWnd>,
    pub data_source: DataSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridFrameWnd {
    pub name: String,
    pub(crate) _d5: SG4,
    //pub(crate) _d5_1: (u32, u32),
    //pub _d5_2: Size, // scroll container size? width bigger, height smaller (or exact, or both bigger)
    //pub(crate) _d5_3: (u32, u32),
    //pub(crate) _d6: u32,
    //pub(crate) _d7_1: SG3,
    pub(crate) cols: SG4,
    //pub(crate) _d7_2: (u32, u32),
    //pub size: Size,
    //pub(crate) _d8_0: u32,
    ///// Probably the number of columns on this table
    //pub col_count: u32,
    ///// Mostly min(col_count, 12)
    //pub cols_shown: u32,
    ////pub(crate) _x0: SG2,
    //pub(crate) _x0_1: SG3,
    pub(crate) keys: SG4,
    //pub(crate) _x0_2: (u32, u32),
    //pub(crate) _x0_3: Size,
    //pub(crate) _e0: u32,
    ///// Possibly the number of key constraints on this table (primary + foreign)
    //pub keys: (u32, u32),
    ////pub(crate) _x1: SG1,
    //pub(crate) _x1_1: SG3,
    pub(crate) x2: SG4,
    //pub(crate) _x1_2: (u32, u32),
    //pub(crate) _x1_3: Size,
    //pub(crate) _x1_4: (u32, u32, u32),
    //pub(crate) _x2: SG1,
    //pub(crate) _x2_1: SG3,
    pub(crate) x3: SG4,
    //pub(crate) _x2_2: (u32, u32),
    //pub(crate) _x2_3: Size,
    //pub(crate) _x2_4: (u32, u32, u32),
    //pub(crate) _x3: SG3,
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
pub(crate) struct SG1(pub(crate) Vec<u32>);

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct SG2(pub(crate) Vec<u32>, pub(crate) Size);

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct SG3 {
    pub(crate) v1: u32,
    pub(crate) v2: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct SG4 {
    v0: (u32, u32),
    v1: Size,
    v2: u32,
    count: u32,
    shown: u32,
    v5: SG3,
}

fn parse_sch_grid_inner4(input: &[u8]) -> IResult<&[u8], SG4> {
    let (input, v0) = le_u32_2(input)?;
    let (input, v1) = Size::parse(input)?;
    let (input, v2) = le_u32(input)?;
    let (input, count) = le_u32(input)?;
    let (input, shown) = le_u32(input)?;
    let (input, v5) = parse_sch_grid_inner3(input)?;
    Ok((
        input,
        SG4 {
            v0,
            v1,
            v2,
            count,
            shown,
            v5,
        },
    ))
}

fn _parse_sch_grid_inner(input: &[u8]) -> IResult<&[u8], SG1> {
    let (input, v1) = count(le_u32, 6)(input)?;
    Ok((input, SG1(v1)))
}

fn _parse_sch_grid_inner2(input: &[u8]) -> IResult<&[u8], SG2> {
    let (input, v1) = count(le_u32, 6)(input)?;
    let (input, size) = Size::parse(input)?;
    Ok((input, SG2(v1, size)))
}

fn parse_sch_grid_inner3(input: &[u8]) -> IResult<&[u8], SG3> {
    let (input, (v2_count, v1)) = pair(le_u32, le_u32)(input)?;
    let (input, v2) = count(le_u32, v2_count as usize)(input)?;
    Ok((input, SG3 { v1, v2 }))
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
    let (input, (v_minor, v_major)) = pair(le_u16, le_u16)(input)?;
    assert_eq!((v_minor, v_major), (7, 0));
    let (input, name) = length_value(le_u32, parse_wstring_nt)(input)?;
    let (input, _d5) = parse_sch_grid_inner4(input)?;
    //let (input, _d7_2) = le_u32_2(input)?;
    //let (input, size) = Size::parse(input)?;
    //let (input, _d8_0) = le_u32(input)?;
    //let (input, col_count) = le_u32(input)?;
    //let (input, cols_shown) = le_u32(input)?;
    //let (input, _x0_1) = parse_sch_grid_inner3(input)?;
    let (input, cols) = parse_sch_grid_inner4(input)?;

    //let (input, _x0_2) = le_u32_2(input)?;
    //let (input, _x0_3) = Size::parse(input)?;
    //let (input, _e0) = le_u32(input)?;
    //let (input, keys) = le_u32_2(input)?;
    //let (input, _x1_1) = parse_sch_grid_inner3(input)?;
    let (input, keys) = parse_sch_grid_inner4(input)?;

    //let (input, _x1_2) = le_u32_2(input)?;
    //let (input, _x1_3) = Size::parse(input)?;
    //let (input, _x1_4) = le_u32_3(input)?;
    //let (input, _x2_1) = parse_sch_grid_inner3(input)?;
    let (input, x2) = parse_sch_grid_inner4(input)?;

    //let (input, _x2_2) = le_u32_2(input)?;
    //let (input, _x2_3) = Size::parse(input)?;
    //let (input, _x2_4) = le_u32_3(input)?;
    //let (input, _x3) = parse_sch_grid_inner3(input)?;
    let (input, x3) = parse_sch_grid_inner4(input)?;

    let (input, data_source) = parse_data_source(input)?;

    Ok((
        input,
        SchGrid {
            extent,
            frame: Box::new(GridFrameWnd {
                name,
                _d5,
                //_d5_1,
                //_d5_2,
                //_d5_3,
                //_d6,
                //_d7_1,
                cols,
                //_d7_2,
                //size,
                //_d8_0,
                //col_count,
                //cols_shown,
                ////_x0,
                //_x0_1,
                keys,
                //_x0_2,
                //_x0_3,
                //_e0,
                //keys,
                //_x1_1,
                x2,
                //_x1_2,
                //_x1_3,
                //_x1_4,
                //_x2_1,
                x3,
                //_x2_2,
                //_x2_3,
                //_x2_4,
                //_x3,
            }),
            data_source,
        },
    ))
}
