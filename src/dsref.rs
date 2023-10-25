use std::borrow::Cow;

use ms_oforms::common::parse_guid;
use nom::{
    combinator::map_opt,
    error::{FromExternalError, ParseError},
    number::complete::{le_u16, le_u32},
    IResult,
};
use uuid::Uuid;

use crate::parse_u32_bytes_wstring_nt;

bitflags::bitflags! {
    /// VS Data Services DsRef Type Enum
    ///
    /// See: <https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.__dsreftype>
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct DsRefType: u32 {
        /// Specifies a collection.
        const COLLECTION = 1;

        /// Specifies a database object.
        const DATABASE = 2048;

        /// Specifies a data source root.
        const DATASOURCEROOT = 16;

        /// Specifies an extended type.
        const EXTENDED = 16384;

        /// Specifies a data field.
        const FIELD = 256;

        /// Specifies a database function.
        const FUNCTION = 16777216;

        /// Specifies that the DSRef object has a first child.
        const HASFIRSTCHILD = 65536;

        /// Specifies that the DSRef object has a moniker.
        const HASMONIKER = 524288;

        /// Specifies that the DSRef object has a name.
        const HASNAME = 262144;

        /// Specifies that the DSRef object has a sibling that can be accessed next.
        const HASNEXTSIBLING = 131072;

        /// Specifies that the DSRef object has an owner.
        const HASOWNER = 2097152;

        /// Specifies that the DSRef object has properties.
        const HASPROP = 4194304;

        /// Specifies a database index.
        const INDEX = 268435456;

        /// Specifies the DSRef object supports mixed mode
        const MIXED = 4;

        /// Specifies a multiple DSRef object.
        const MULTIPLE = 2;

        /// Specifies a generic node.
        const NODE = 0xFF90FF00;  //-7274752;

        /// Specifies a null value (0).
        const NULL = 0;

        /// Specifies a package.
        const PACKAGE = 33554432;

        /// Specifies a package body.
        const PACKAGEBODY = 67108864;

        /// Specifies a query.
        const QUERY = 1024;

        /// Specifies a database relationship object.
        const RELATIONSHIP = 134217728;

        /// The DSRef object.
        const SCHEMADIAGRAM = 32768;

        /// Specifies a stored procedure.
        const STOREDPROCEDURE = 8192;

        /// Specifies a synonym.
        const SYNONYM = 8388608;

        /// Specifies a table.
        const TABLE = 512;

        /// Specifies a trigger.
        const TRIGGER = 4096;

        /// Specifies a user-defined type.
        const USERDEFINEDTYPE = 536870912;

        /// The DSRef object.
        const VIEW = 1048576;

        /// Specifies a database view index.
        const VIEWINDEX = 0x80000000; //-2147483648;

        /// Specifies a database view trigger.
        const VIEWTRIGGER = 1073741824;

    }
}

#[derive(Debug, Clone)]
pub struct DSRefSchemaEntry {
    pub ref_type: DsRefType,
    pub table: String,
    pub schema: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DSRefSchemaContents {
    pub clsid: Uuid,
    pub(crate) len: usize,
    pub(crate) a: Uuid, // probably not actually a UUID
    pub root_ref: DsRefType,
    pub(crate) _d1: Uuid, // NIL UUID
    pub connection: String,
    pub ref_type: DsRefType,
    pub name: String,
    pub tables: Vec<DSRefSchemaEntry>,
    pub(crate) c: u32,
    pub(crate) c2: Uuid,
    pub(crate) c3: u16, // could be tagVARENUM? 0x0008 is BSTR
    pub guid: String,
}

fn parse_entry<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], DSRefSchemaEntry, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, ref_type) = map_opt(le_u32, DsRefType::from_bits)(input)?;
    let (input, table) = parse_u32_bytes_wstring_nt(input)?;
    let (input, schema) = parse_u32_bytes_wstring_nt(input)?;
    Ok((
        input,
        DSRefSchemaEntry {
            ref_type,
            table,
            schema,
        },
    ))
}

pub fn parse_dsref_schema_contents<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], DSRefSchemaContents, E>
where
    E: ParseError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, clsid) = parse_guid(input)?;
    let len = input.len();
    let (input, a) = parse_guid(input)?;
    let (input, root_ref) = map_opt(le_u32, DsRefType::from_bits)(input)?;
    let (input, _d1) = parse_guid(input)?;
    let (input, connection) = parse_u32_bytes_wstring_nt(input)?;
    let (input, ref_type) = map_opt(le_u32, DsRefType::from_bits)(input)?;
    let (input, name) = if ref_type.contains(DsRefType::HASNAME) {
        parse_u32_bytes_wstring_nt(input)?
    } else {
        (input, String::new())
    };
    let (input, tables) = {
        let mut tables = Vec::new();
        let mut hasnext = ref_type.contains(DsRefType::HASFIRSTCHILD);
        let mut _i = input;
        while hasnext {
            let (rest, entry) = parse_entry(_i)?;
            hasnext = entry.ref_type.contains(DsRefType::HASNEXTSIBLING);
            tables.push(entry);
            _i = rest;
        }
        (_i, tables)
    };
    let (input, c) = le_u32(input)?;
    let (input, c2) = parse_guid(input)?;
    let (input, c3) = le_u16(input)?;
    let (input, guid) = parse_u32_bytes_wstring_nt(input)?;
    Ok((
        input,
        DSRefSchemaContents {
            clsid,
            len,
            a,
            root_ref,
            _d1,
            connection,
            ref_type,
            name,
            tables,
            c,
            c2, // b30985d6-6bbb-45f2-9ab8-371664f03270 ?
            c3, //
            guid,
        },
    ))
}
