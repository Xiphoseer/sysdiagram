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
use ms_oforms::{
    controls::{
        form::{FormControl, Site},
        ole_site_concrete::Clsid,
    },
    OFormsFile,
};
use nom::error::VerboseError;
pub use parser::*;
mod dsref;
pub use dsref::*;
mod connection_string;
pub use connection_string::*;
use uuid::{uuid, Uuid};

const DSREF_SCHEMA_CONTENTS: &str = "/DSREF-SCHEMA-CONTENTS";

// See: http://www.dejadejadeja.com/detech/ocxdb/guidintnames.txt
pub const CLSID_SCHGRID: Uuid = uuid!("e9b0e6d9-811c-11d0-ad51-00a0c90f5739");
pub const CLSID_POLYLINE: Uuid = uuid!("d24d4453-1f01-11d1-8e63-006097d2df48");
pub const CLSID_DDSLABEL: Uuid = uuid!("d24d4451-1f01-11d1-8e63-006097d2df48");

pub struct SysDiagramFile<T> {
    inner: OFormsFile<T>,
}

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

    pub fn schema_form(
        &mut self,
    ) -> Result<(FormControl, Vec<Table>, Vec<Relationship>), LoadError> {
        eprintln!("Parsing FormControl");
        let form_control = self.root_form_control().map_err(LoadError::Cfb)?;

        eprintln!("Parsing Objects");
        if self.is_stream("/o") {
            let mut o_stream = self.root_object_stream().map_err(LoadError::Cfb)?;
            let o_stream_len = usize::try_from(o_stream.len()).map_err(LoadError::StreamTooLong)?;
            let mut bytes: Vec<u8> = Vec::with_capacity(o_stream_len);
            o_stream.read_to_end(&mut bytes).map_err(LoadError::Cfb)?;

            let mut offset = 0;
            let mut tables = Vec::new();
            let mut relationships = Vec::new();
            for site in &form_control.sites[..] {
                match site {
                    Site::Ole(ref ole_site) => {
                        let site_len = usize::try_from(ole_site.object_stream_size)
                            .map_err(LoadError::SiteTooLong)?;
                        match ole_site.clsid_cache_index {
                            Clsid::ClassTable(index) => {
                                let caption = ole_site.control_tip_text.clone();
                                let data = &bytes[offset..];
                                if index == 0 {
                                    // Table
                                    let (_, sch_grid) = parser::parse_sch_grid(data)?;
                                    tables.push(Table { sch_grid, caption });
                                } else if index == 1 {
                                    // Foreign Key
                                    let (_, control) = parser::parse_control1(data)?;
                                    let (_, (name, from, to)) =
                                        parser::parse_relationship(&caption[..])?;
                                    relationships.push(Relationship {
                                        control,
                                        caption,
                                        name,
                                        from,
                                        to,
                                    });
                                } else if index == 2 {
                                    // Control?
                                    // TODO
                                }
                            }
                            Clsid::Invalid => println!("Invalid Class"),
                            Clsid::Global(index) => println!("GLOBAL {}", index),
                        };
                        offset += site_len;
                    }
                }
            }
            Ok((form_control, tables, relationships))
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
