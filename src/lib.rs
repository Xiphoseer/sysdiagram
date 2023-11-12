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
//! ## Installation
//!
//! To install the debugging tool that comes with this crate, run
//!
//! ```sh
//! cargo install --git https://github.com/Xiphoseer/sysdiagram.git -F cli
//! ```
//!
//! ## File Format
//!
//! On the surface, the binary blob of the sysdiagram is a OLE 2.0 embedded object
//! as specified by [\[MS-OLEDS\]] and [\[MS-CFB\]]. Specifically it has a `\1CompObj` stream
//! that specifies a clipboard format, but no `\1Ole` (or `\1Ole10Native`) stream.
//!
//! The clipboard format is `Embedded Object` with a user type of `Microsoft DDS Form 2.0`.
//! That matches [ProgID] `MSDDS.Form.080.1` ([CLSID `{77D2C92E-7779-11D8-9070-00065B840D9C}`][`dds::CLSID_DDS_FORM`])
//! from `msddsf.dll`. That refers to the *Microsoft Design Tools - DDS Forms*. *DDS* in this case stands for
//! [*DaVinci Design Surface*][`dds`], which is a set of OLE Controls originating in Visual Studio
//! to implement visual designers.
//!
//! Aside from the database designer, it was also used in the "Query Designer", "Table Designer", and a "Site Designer".
//! A variant still appears in XML in the DTSX [`dts-designer-1.0`] namespaced PackageVariable, which uses `<dds>` elements
//! in a unspecified `xmlns:dwd="http://schemas.microsoft.com/DataWarehouse/Designer/1.0"` namespace to store layout
//! information.
//!
//! Most importantly though, it was used in the Visual Studio 5.0 and 6.0 for the UserForm
//! designer, which also found its way into Office via the [VBA UserForm]s
//! and their [Designer Storages](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/f614ae64-1b3d-47be-a166-0e10b8230026).
//!
//! There, the form part became [\[MS-OFORMS\]], which specifies the binary file format used
//! in the `f` stream to encode a UserForm control. This control contains a sequence of OLE control
//! sites and positions as well as a table of CLSIDs of the used controls and lengths within the `o`
//! stream where the actual controls are persisted.
//!
//! The compound object also has a `DSREF-SCHEMA-CONTENTS` stream, which is a persisted
//! [Data Source Reference Object (DSRef)][`dsref`] and probably makes it be a `CF_DSREF`
//! clipboard object as well. A DSRef is a Visual Studio abstraction that makes it possible
//! to drag & drop "data objects" between tools & hierarchies. The DSRef in a sysdiagram has:
//! - a root note of type `DATABASE` (name is a connection string)
//! - with one child node of type `SCHEMADIAGRAM` (name is the diagram name)
//! - one child node of type `TABLE` per table in the diagram (name is table name, owner is schema)
//!
//! ## The Controls
//!
//! Tables are represented by a [`SchGrid`] OLE Control, which is bound to a table of
//! the Data Source via the DSRef. In the designer, each row of this grid control represents
//! a column of a table.
//!
//! Foreign key relationships are represented by [`dds::Polyline`]s with tooltips and associated [`dds::Label`]s.
//!
//! ## Preview
//!
//! ![Database Diagram](https://raw.githubusercontent.com/Xiphoseer/sysdiagram/ad596ad4e17bf25e6e004a212c1d12d03c97f28e/res/dv3w7c1.gif)
//!
//! [\[MS-OFORMS\]]: https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms
//! [\[MS-OLEDS\]]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-oleds
//! [\[MS-CFB\]]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb
//! [ProgID]: https://learn.microsoft.com/en-us/windows/win32/com/-progid--key
//! [VBA UserForm]: https://learn.microsoft.com/en-us/office/vba/excel/concepts/controls-dialogboxes-forms/create-a-user-form
//! [`dts-designer-1.0`]: https://learn.microsoft.com/en-us/openspecs/sql_data_portability/ms-dtsx/a7d84cd1-4aca-433a-b450-58b331fca519

mod core;
pub use core::*;
use std::{
    convert::TryFrom,
    io::{Read, Seek},
    ops::DerefMut,
};
mod dtyp;
mod error;
pub use error::*;
pub mod mdtdb;
mod parser;
pub use mdtdb::SchGrid;
use ms_oforms::{
    controls::user_form::FormControl, properties::FormEmbeddedActiveXControl, OFormsFile,
};
use nom::error::VerboseError;
pub use parser::*;
mod connection_string;
pub mod dds;
pub mod dsref;
pub use connection_string::*;
use dsref::{parse_dsref_schema_contents, DSRefSchemaContents};

use crate::{
    dds::{parse_label, parse_polyline, CLSID_DDSLABEL, CLSID_POLYLINE},
    mdtdb::{parse_sch_grid, CLSID_SCHGRID},
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
