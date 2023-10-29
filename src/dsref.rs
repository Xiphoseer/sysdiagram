use std::{borrow::Cow, collections::BTreeMap};

use ms_oforms::common::{parse_guid, VarType};
use nom::{
    combinator::{cond, map, map_opt},
    error::{ContextError, FromExternalError, ParseError},
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
pub struct DsRefNode {
    pub flags: DsRefType,
    pub extended_type: Option<Uuid>,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub children: Vec<DsRefNode>,
    pub properties: Option<BTreeMap<Uuid, Variant>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DSRefSchemaContents {
    pub clsid: Uuid,
    pub(crate) len: usize,
    pub(crate) a: Uuid, // probably not actually a UUID
    pub root_node: DsRefNode,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Variant {
    BStr(String),
}

fn parse_dsref_properties<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], BTreeMap<Uuid, Variant>, E>
where
    E: ParseError<&'a [u8]>,
    E: ContextError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, prop_count) = le_u32(input)?;
    let mut _i = input;
    let mut prop_map = BTreeMap::new();
    for _index in 0..prop_count {
        let input = _i;
        let (input, property) = parse_guid(input)?;
        let (input, vt) = map_opt(le_u16, VarType::from_bits)(input)?;
        let (input, value) = match vt {
            VarType::BSTR => map(parse_u32_bytes_wstring_nt, Variant::BStr)(input),
            _ => todo!(),
        }?;
        prop_map.insert(property, value);
        _i = input;
    }
    Ok((_i, prop_map))
}

pub fn parse_dsref_node<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], DsRefNode, E>
where
    E: ParseError<&'a [u8]>,
    E: ContextError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, flags) = map_opt(le_u32, DsRefType::from_bits)(input)?;
    let (input, extended_type) = cond(flags.contains(DsRefType::EXTENDED), parse_guid)(input)?;
    let (input, name) = cond(
        flags.contains(DsRefType::HASNAME),
        parse_u32_bytes_wstring_nt,
    )(input)?;
    let (input, owner) = cond(
        flags.contains(DsRefType::HASOWNER),
        parse_u32_bytes_wstring_nt,
    )(input)?;
    let (input, children) = {
        let mut nodes = Vec::new();
        let mut hasnext = flags.contains(DsRefType::HASFIRSTCHILD);
        let mut _i = input;
        while hasnext {
            let (rest, entry) = parse_dsref_node(_i)?;
            hasnext = entry.flags.contains(DsRefType::HASNEXTSIBLING);
            nodes.push(entry);
            _i = rest;
        }
        (_i, nodes)
    };
    let (input, properties) =
        cond(flags.contains(DsRefType::HASPROP), parse_dsref_properties)(input)?;
    Ok((
        input,
        DsRefNode {
            flags,
            extended_type,
            name,
            owner,
            children,
            properties,
        },
    ))
}

pub fn parse_dsref_schema_contents<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], DSRefSchemaContents, E>
where
    E: ParseError<&'a [u8]>,
    E: ContextError<&'a [u8]>,
    E: FromExternalError<&'a [u8], Cow<'static, str>>,
{
    let (input, clsid) = parse_guid(input)?;
    let len = input.len();
    let (input, a) = parse_guid(input)?;
    let (input, root_node) = parse_dsref_node(input)?;
    Ok((
        input,
        DSRefSchemaContents {
            clsid,
            len,
            a,
            root_node,
        },
    ))
}
