//! # Reading MSSQL sysdiagrams
//!
//! This crate documents and implements parsers for the binary format used in
//! the `definition` column of the `[dbo].[sysdiagrams]` system table in
//! Microsoft SQL Server 2008.
//!
//! These represent "Database Diagrams" created in SQL Server Management Studio (SMSS),
//! or rather its "Visual Database Tools" component. Diagrams from that era encode
//! a graphical representation of an ER Diagram, that is a set of tables and relations,
//! but the binary format stores next to no information on the columns themselves.
//!
//! Tables are represented by a [`SchGrid`] OLE Control, which is bound to a table of
//! the Data Source via the [Data Source Reference Object (DSRef)][`dsref`]. In the designer,
//! each row of this grid control represents a column of a table.
//!
//! ## Technical Underpinnings
//!
//! The "Microsoft Data Tools Database Designer" alongside its siblings, the "Microsoft Data Tools Query Designer"
//! and "Microsoft Data Tools Table Designer" were implemented using the "Microsoft DT DDS Form 2.0"
//! technology. *DDS* in this case stands for [*DaVinci Diagram Surface*][`dds`], which is a set of OLE
//! Controls originating in Visual Studio.
//!
//! ## Preview
//!
//! ![Database Diagram](https://raw.githubusercontent.com/Xiphoseer/sysdiagram/ad596ad4e17bf25e6e004a212c1d12d03c97f28e/res/dv3w7c1.gif)

mod core;
pub use core::*;
use std::{
    convert::TryFrom,
    io::{Read, Seek},
    ops::DerefMut,
};
mod error;
pub use error::*;
mod parser;
pub mod schgrid;
use ms_oforms::{
    controls::user_form::FormControl, properties::FormEmbeddedActiveXControl, OFormsFile,
};
use nom::error::VerboseError;
pub use parser::*;
pub use schgrid::SchGrid;
mod connection_string;
pub mod dds;
pub mod dsref;
pub use connection_string::*;
use dsref::{parse_dsref_schema_contents, DSRefSchemaContents};

use crate::{
    dds::{parse_label, parse_polyline, CLSID_DDSLABEL, CLSID_POLYLINE},
    schgrid::{parse_sch_grid, CLSID_SCHGRID},
};

const DSREF_SCHEMA_CONTENTS: &str = "/DSREF-SCHEMA-CONTENTS";

// See: http://www.dejadejadeja.com/detech/ocxdb/
// See: http://pitcheploy.free.fr/Microsoft%20Visual%20Basic%206.0%20%C3%89dition%20Professionnelle%20(Fran%C3%A7ais)/HKEY_LOCAL_MACHINE.txt
// See: https://gist.githubusercontent.com/stevemk14ebr/af8053c506ef895cd520f8017a81f913/raw/98944bc6ae995229d5231568a8ae73dd287e8b4f/guids
// See: https://gist.githubusercontent.com/hfiref0x/a77584e47b0feb3779f47c8d7609d4c4/raw/0cedbcaee37c072c623c71c2b2ac03ab020592da/responder_comdata.txt

pub struct SysDiagramFile<T> {
    inner: OFormsFile<T>,
}

type SchemaForm = (FormControl, Vec<(SiteInfo, Control)>);

impl<T: Read + Seek> SysDiagramFile<T> {
    pub fn open(inner: T) -> std::io::Result<Self> {
        let inner = OFormsFile::open(inner)?;
        Ok(Self { inner })
    }

    pub fn dsref_schema_contents_stream(&mut self) -> std::io::Result<cfb::Stream<T>> {
        self.inner.open_stream(DSREF_SCHEMA_CONTENTS)
    }

    pub fn dsref_schema_contents(&mut self) -> Result<DSRefSchemaContents, Error> {
        if self.is_stream(DSREF_SCHEMA_CONTENTS) {
            let mut r_stream = self.dsref_schema_contents_stream().map_err(Error::Cfb)?;
            let r_stream_len = usize::try_from(r_stream.len()).map_err(Error::StreamTooLong)?;
            let mut bytes: Vec<u8> = Vec::with_capacity(r_stream_len);
            r_stream.read_to_end(&mut bytes).map_err(Error::Cfb)?;
            let (_, dsref_schema_contents) =
                parse_dsref_schema_contents::<VerboseError<_>>(&bytes[..])?;
            Ok(dsref_schema_contents)
        } else {
            Err(Error::MissingStream(DSREF_SCHEMA_CONTENTS))
        }
    }

    pub fn schema_form(&mut self) -> Result<SchemaForm, Error> {
        if !self.is_stream("/f") {
            return Err(Error::MissingStream("f"));
        }
        eprintln!("Parsing FormControl");
        let mut form = self.root_form().map_err(Error::Cfb)?;

        if !self.is_stream("/o") {
            return Err(Error::MissingStream("o"));
        }
        eprintln!("Parsing Objects");

        let mut iter = form.site_iter();

        let mut controls = Vec::new();

        let mut buf = Vec::<u8>::new();
        while let Some((ctrl_class, depth, ole_site)) = iter.next() {
            let site_len = ole_site.object_stream_size as usize;

            buf.truncate(0); // reset len, keep capacity
            buf.reserve(site_len);
            let mut s = iter.site_stream().map_err(Error::Cfb)?;
            s.read_to_end(&mut buf)?;
            let data = &buf[..];

            //println!("{:>3} (len: {:>4}) {}: {} ", i, site_len, clsid, caption);
            //println!("{:?}", ole_site.site_position);
            let clsid = match ctrl_class {
                FormEmbeddedActiveXControl::ControlNonCached(class_info) => class_info.cls_id,
                FormEmbeddedActiveXControl::ControlCached(_) => unimplemented!(""),
            };
            let control = match clsid {
                CLSID_SCHGRID => {
                    // Table
                    let (_, sch_grid) = parse_sch_grid(data)?;
                    Control::SchGrid(sch_grid)
                }
                CLSID_POLYLINE => {
                    // Foreign Key
                    let (_, control) = parse_polyline(data)?;
                    //let (_, (name, from, to)) = parser::parse_relationship(&caption[..])?;
                    Control::Polyline(control)
                }
                CLSID_DDSLABEL => {
                    let (_, label) = parse_label::<nom::error::Error<_>>(data)?;
                    Control::Label(label)
                }
                _ => {
                    eprintln!("Unknown clsid: {}", clsid);
                    Control::Unknown(clsid)
                }
            };
            controls.push((
                SiteInfo {
                    id: ole_site.id,
                    depth,
                    pos: ole_site.site_position,
                    tooltip: ole_site.control_tip_text.clone(),
                },
                control,
            ))
        }
        let form_control = form.into_form_control();
        Ok((form_control, controls))
    }
}

impl<T> std::ops::Deref for SysDiagramFile<T> {
    type Target = OFormsFile<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for SysDiagramFile<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
