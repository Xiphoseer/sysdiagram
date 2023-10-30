//! # Reading MSSQL sysdiagrams

mod core;
pub use core::*;
use std::{
    convert::TryFrom,
    io::{Read, Seek},
    ops::DerefMut,
};
mod io;
pub use io::*;
mod parser;
use ms_oforms::{controls::form::FormControl, OFormsFile};
use nom::error::VerboseError;
pub use parser::*;
mod connection_string;
pub mod dds;
pub mod dsref;
pub use connection_string::*;
use dsref::{parse_dsref_schema_contents, DSRefSchemaContents};
use uuid::{uuid, Uuid};

const DSREF_SCHEMA_CONTENTS: &str = "/DSREF-SCHEMA-CONTENTS";

// See: http://www.dejadejadeja.com/detech/ocxdb/
// See: http://pitcheploy.free.fr/Microsoft%20Visual%20Basic%206.0%20%C3%89dition%20Professionnelle%20(Fran%C3%A7ais)/HKEY_LOCAL_MACHINE.txt
// See: https://gist.githubusercontent.com/stevemk14ebr/af8053c506ef895cd520f8017a81f913/raw/98944bc6ae995229d5231568a8ae73dd287e8b4f/guids
// See: https://gist.githubusercontent.com/hfiref0x/a77584e47b0feb3779f47c8d7609d4c4/raw/0cedbcaee37c072c623c71c2b2ac03ab020592da/responder_comdata.txt

/// `SchGrid OLE Custom Control module` (`mdt2db.dll`)
pub const TYPELIB_SCHGRID: Uuid = uuid!("e9b0e6da-811c-11d0-ad51-00a0c90f5739");

/// Microsoft Data Tools Database Designer
pub const CLSID_MSDTDB_DESIGNER: Uuid = uuid!("e9b0e6d4-811c-11d0-ad51-00a0c90f5739");
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

/// Microsoft DT DDSForm (`mdt2df.dll`)
pub const TYPELIB_DDS_FORM: Uuid = uuid!("105b80d0-95f1-11d0-b0a0-00aa00bdcb5c");

/// Microsoft DT DDSform 2.1 FormPackage
pub const CLSID_DDS2_FORM_PACKAGE: Uuid = uuid!("105b80d5-95f1-11d0-b0a0-00aa00bdcb5c");

pub struct SysDiagramFile<T> {
    inner: OFormsFile<T>,
}

type SchemaForm = (FormControl, Vec<Table>, Vec<Relationship>, Vec<Label>);

impl<T: Read + Seek> SysDiagramFile<T> {
    pub fn open(inner: T) -> std::io::Result<Self> {
        let inner = OFormsFile::open(inner)?;
        Ok(Self { inner })
    }

    pub fn dsref_schema_contents_stream(&mut self) -> std::io::Result<cfb::Stream<T>> {
        self.inner.open_stream(DSREF_SCHEMA_CONTENTS)
    }

    pub fn dsref_schema_contents(&mut self) -> Result<DSRefSchemaContents, LoadError> {
        if self.is_stream(DSREF_SCHEMA_CONTENTS) {
            let mut r_stream = self
                .dsref_schema_contents_stream()
                .map_err(LoadError::Cfb)?;
            let r_stream_len = usize::try_from(r_stream.len()).map_err(LoadError::StreamTooLong)?;
            let mut bytes: Vec<u8> = Vec::with_capacity(r_stream_len);
            r_stream.read_to_end(&mut bytes).map_err(LoadError::Cfb)?;
            let (_, dsref_schema_contents) =
                parse_dsref_schema_contents::<VerboseError<_>>(&bytes[..])?;
            Ok(dsref_schema_contents)
        } else {
            Err(LoadError::MissingStream(DSREF_SCHEMA_CONTENTS))
        }
    }

    pub fn schema_form(&mut self) -> Result<SchemaForm, LoadError> {
        eprintln!("Parsing FormControl");
        let form_control = self.root_form_control().map_err(LoadError::Cfb)?;
        println!("{:?}", form_control.displayed_size);

        eprintln!("Parsing Objects");
        if self.is_stream("/o") {
            let mut form = self.root_form().map_err(LoadError::Cfb)?;
            let mut iter = form.site_iter();

            let mut tables = Vec::new();
            let mut relationships = Vec::new();
            let mut labels = Vec::new();

            let mut buf = Vec::<u8>::new();
            let mut _i = 0;
            while let Some((clsid, ole_site)) = iter.next() {
                let site_len = ole_site.object_stream_size as usize;
                let caption = ole_site.control_tip_text.clone();

                buf.truncate(0); // reset len, keep capacity
                buf.reserve(site_len);
                let mut s = iter.site_stream().map_err(LoadError::Cfb)?;
                s.read_to_end(&mut buf)?;
                let data = &buf[..];

                //println!("{:>3} (len: {:>4}) {}: {} ", i, site_len, clsid, caption);
                //println!("{:?}", ole_site.site_position);
                match clsid {
                    CLSID_SCHGRID => {
                        // Table
                        let (_, sch_grid) = parser::parse_sch_grid(data)?;
                        tables.push(Table {
                            _index: _i,
                            sch_grid,
                            caption,
                        });
                    }
                    CLSID_POLYLINE => {
                        // Foreign Key
                        let (_, control) = parser::parse_polyline(data)?;
                        let (_, (name, from, to)) = parser::parse_relationship(&caption[..])?;
                        relationships.push(Relationship {
                            _index: _i,
                            control,
                            caption,
                            name,
                            from,
                            to,
                        });
                    }
                    CLSID_DDSLABEL => {
                        let (_, label) = parser::parse_label::<nom::error::Error<_>>(data)?;
                        labels.push(label);
                    }
                    _ => eprintln!("Unknown clsid: {}", clsid),
                }

                _i += 1;
            }
            Ok((form_control, tables, relationships, labels))
        } else {
            Err(LoadError::MissingStream("o"))
        }
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
