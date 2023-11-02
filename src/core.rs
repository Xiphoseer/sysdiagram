#![allow(clippy::upper_case_acronyms)]
//! # Data definitions for sysdiagrams
use ms_oforms::properties::{Position, Size};
use uuid::Uuid;

use crate::{
    dds::{Label, Polyline},
    DSRefSchemaContents,
};

/// ## SchGrid Control
///
/// See: <http://www.dejadejadeja.com/detech/ocxdb/mdt2db.dll.txt.lisp>
#[derive(Debug)]
#[allow(dead_code)]
pub struct SchGrid {
    pub(crate) d1: u32,
    pub(crate) d2: u32,
    pub size1: Size,
    pub(crate) d3: u32,
    pub(crate) d4: u32,
    pub name: String,
    pub(crate) d5_1: (u32, u32),
    pub(crate) d5_2: (u32, u32), // pos/size?
    pub(crate) d5_3: (u32, u32),
    pub(crate) d6: u32,
    pub(crate) _d7: Vec<u32>,
    pub(crate) size2: Size,
    pub(crate) d8_0: u32,
    pub col_count: u32,
    pub cols_shown: u32, // mostly min(col_count, 12)
    //pub(crate) d8: Vec<u32>,
    //pub(crate) d9: u32,
    pub(crate) x1: Vec<SchGridInner>,
    //pub(crate) d10: Vec<u32>,
    //pub(crate) d11: BString,
    //pub(crate) d12: u32,
    //pub(crate) d13: (u32, u32), // border width? 1,1
    pub(crate) d14: Vec<u32>, // 0 - 10, selected columns?
    pub table: String,
    pub schema: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct SchGridInner(pub(crate) Vec<u32>);

#[derive(Debug)]
pub struct SiteInfo {
    pub id: i32,
    pub pos: Position,
    pub tooltip: String,
}

#[derive(Debug)]
pub enum Control {
    SchGrid(SchGrid),
    Label(Label),
    Polyline(Polyline),
    Unknown(Uuid),
}

#[derive(Debug)]
pub struct Table {
    pub id: i32,
    pub sch_grid: SchGrid,
    pub caption: String,
}

#[derive(Debug)]
pub struct Relationship {
    pub id: i32,
    pub control: Polyline,
    pub caption: String,
    pub from: String,
    pub to: String,
    pub name: String,
}

#[derive(Debug)]
pub struct SysDiagram {
    pub tables: Vec<Table>,
    pub relationships: Vec<Relationship>,
    pub dsref_schema_contents: DSRefSchemaContents,
}
