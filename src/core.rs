#![allow(clippy::upper_case_acronyms)]
//! # Data definitions for sysdiagrams
use bstr::BString;
use ms_oforms::properties::{font::StdFont, Position, Size};

use crate::DSRefSchemaContents;

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
    pub(crate) d7: Vec<u32>,
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
    pub(crate) _d2: u32,
    pub(crate) _d3: u32,
    pub(crate) _b1: u8,
    pub(crate) _d4: i32,
    pub(crate) _d5: i32,
    pub(crate) _d6: i32,
    pub(crate) _x1: BString,
    pub font: StdFont,
    pub text: String,
}

#[derive(Debug)]
pub struct Table {
    pub(crate) _index: usize,
    pub sch_grid: SchGrid,
    pub caption: String,
}

#[derive(Debug)]
pub struct Relationship {
    pub(crate) _index: usize,
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
