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
    pub a: SchGridA,
    pub b: SchGridB,
    pub c: SchGridC,
}

#[derive(Debug)]
pub struct SchGridA {
    pub(crate) _d2: u32,
    pub size1: Size,
}

#[derive(Debug)]
pub struct SchGridB {
    pub(crate) _d4: u32,
    pub name: String,
    pub(crate) _d5_1: (u32, u32),
    pub _d5_2: Size, // scroll container size? width bigger, height smaller (or exact, or both bigger)
    pub(crate) _d5_3: (u32, u32),
    pub(crate) _d6: u32,
    pub(crate) _d7: Vec<u32>,
    pub(crate) _size2: Size,
    pub(crate) _d8_0: u32,
    pub col_count: u32,
    pub cols_shown: u32, // mostly min(col_count, 12)
    //pub(crate) d8: Vec<u32>,
    //pub(crate) d9: u32,
    pub(crate) _x1: Vec<SchGridInner>,
    pub(crate) _x2: Vec<u32>,
}

#[derive(Debug)]
pub struct SchGridC {
    pub(crate) _cd1: u32,
    pub(crate) _cd2: u32,
    pub(crate) _cd3: u32,
    pub(crate) _cd4: u32,
    pub(crate) _d14: Vec<u32>, // 0 - 10, selected columns?
    pub table: String,
    pub schema: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct SchGridInner(pub(crate) Vec<u32>, pub(crate) Size, pub(crate) Vec<u32>);

#[derive(Debug)]
pub struct SiteInfo {
    pub id: i32,
    pub depth: u8,
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
