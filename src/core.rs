#![allow(clippy::upper_case_acronyms)]
//! # Data definitions for sysdiagrams
use ms_oforms::properties::Position;
use uuid::Uuid;

use crate::{
    dds::{Label, Polyline},
    schgrid::SchGrid,
    DSRefSchemaContents,
};

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
