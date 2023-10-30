//! # Data Source Reference Object
//!
//! The `DSRef` object is a tree of nodes that represents a reference to a data source.
//!
//! Each [node][`DsRefNode`] can have:
//!
//! - A [name][`DsRefNode::name`] string, e.g. the name of the table (see [`HASNAME`][`DsRefType::HASNAME`], [`GetName`], [`SetName`])
//! - A [owner][`DsRefNode::owner`] string, e.g. the schema of the table (see [`HASOWNER`][`DsRefType::HASOWNER`], [`GetOwner`], [`SetOwner`])
//! - An [extended type][`DsRefNode::extended_type`] GUID
//!   - see [`EXTENDED`][`DsRefType::EXTENDED`], [`GetExtendedType`], [`SetExtendedType`]
//!   - Currently, only the nil GUID `00000000-0000-0000-0000-0000000000000000` is supported
//! - A sequence of [children][`DsRefNode::children`]
//!   - see [`HASFIRSTCHILD`][`DsRefType::HASFIRSTCHILD`], [`GetFirstChildNode`], [`SetFirstChildNode`]
//!   - see [`HASNEXTSIBLING`][`DsRefType::HASNEXTSIBLING`], [`GetNextSiblingNode`], [`SetNextSiblingNode`]
//! - A set of [GUID][`Uuid`]-keyed, [`Variant`]-valued [properties][`DsRefNode::properties`] (see [`HASPROP`][`DsRefType::HASPROP`], [`GetProperty`], [`SetProperty`])
//!
//! Unimplemented:
//!
//! - A *moniker* (see [`HASMONIKER`][`DsRefType::HASMONIKER`], [`GetMoniker`], [`SetMoniker`])
//!
//! ## Implementations in .NET:
//! - [`Microsoft.VisualStudio.Data.Interop` Namespace](https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.interop)
//! - [`Microsoft.VisualStudio.Data.Services.SupportEntities.Interop` Namespace](https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop)
//! - `Microsoft.VisualStudio.Data.Framework` Namespace:
//!   - [`DSRefBuilder` Class](https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.framework.dsrefbuilder)
//! - `Microsoft.SqlServer.Management.UI.VSIntegration` Namespace:
//!   - [`VsDataSupport` Class](https://learn.microsoft.com/en-us/dotnet/api/microsoft.sqlserver.management.ui.vsintegration.vsdatasupport)
//! - `Microsoft.ReportDesigner.Data.Server.Dialogs.Pages` Namespace
//!   - [`IDataSourceGeneralPage.SelectedDSRef` Property](https://learn.microsoft.com/en-us/dotnet/api/microsoft.reportdesigner.data.server.dialogs.pages.idatasourcegeneralpage.selecteddsref)
//!
//! [`GetName`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefconsumer.getname
//! [`SetName`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefprovider.setname
//! [`GetOwner`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefconsumer.getowner
//! [`SetOwner`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefprovider.setowner
//! [`GetMoniker`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefconsumer.getmoniker
//! [`SetMoniker`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefprovider.setmoniker
//! [`GetExtendedType`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefconsumer.getextendedtype
//! [`SetExtendedType`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefprovider.setextendedtype
//! [`GetFirstChildNode`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefconsumer.getfirstchildnode
//! [`SetFirstChildNode`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefprovider.setfirstchildnode
//! [`GetNextSiblingNode`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefconsumer.getnextsiblingnode
//! [`SetNextSiblingNode`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefprovider.setnextsiblingnode
//! [`GetProperty`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefconsumer.getproperty
//! [`SetProperty`]: https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.data.services.supportentities.interop.idsrefprovider.setproperty

use crate::parse_u32_bytes_wstring_nt;
use ms_oforms::common::{parse_guid, VarType};
use nom::{
    combinator::{cond, map, map_opt},
    error::{ContextError, FromExternalError, ParseError},
    number::complete::{le_u16, le_u32},
    IResult,
};
use std::{borrow::Cow, collections::BTreeMap};
use uuid::{uuid, Uuid};

/// Microsoft Data Tools DSRef Object `{e9b0e6db-811c-11d0-ad51-00a0c90f5739}`
///
/// (aka `DSRefObject2.Simple`, from `mdt2fref.dll`)
pub const CLSID_DSREF_R2: Uuid = uuid!("e9b0e6db-811c-11d0-ad51-00a0c90f5739");

pub const CLSID_DSREF_R1: Uuid = uuid!("ab36de40-2bf4-11ce-ab3c-00aa004404fb");

/// Microsoft Visual Studio Data DSRef Object `{e09ee6ac-fef0-41ae-9f77-3c394da49849}`
pub const CLSID_DSREF: Uuid = uuid!("e09ee6ac-fef0-41ae-9f77-3c394da49849");
/// Microsoft Data Tools DSRef Object `DSRefObject2.Simple`
pub const PROGID_DSREF_R2: &str = "DSRefObject2.Simple";

// https://github.com/BlackbirdSQL/Firebird-DDEX-SqlEditor/blob/111af4915f189fe48b4326c07c4c649815ed6670/BlackbirdSql.Core/Root/VS.cs#L42
/// `GUID_DSRefProperty_Provider` (GUID BSTR) `{b30985d6-6bbb-45f2-9ab8-371664f03270}`
pub const GUID_DSREF_PROPERTY_PROVIDER: Uuid = uuid!("b30985d6-6bbb-45f2-9ab8-371664f03270");
/// `GUID_DSRefProperty_PreciseType` (int32) `{39a5a7e7-513f-44a4-b79d-7652cd8962d9}`
pub const GUID_DSREF_PROPERTY_PRECISE_TYPE: Uuid = uuid!("39a5a7e7-513f-44a4-b79d-7652cd8962d9");

pub const IID_IDSREF_CONSUMER: Uuid = uuid!("AB36DE42-2BF4-11CE-AB3C-00AA004404FB");
pub const IID_IDSREF_PROVIDER: Uuid = uuid!("AB36DE41-2BF4-11CE-AB3C-00AA004404FB");

// https://github.com/adityachandra1/MIT-Cafeteria-DBS/blob/ac3a7a915a427a42035c56592dfe0c73932ae669/src/server/microsoft-sql-server/SqlDbTools.pkgdef#L378
/// .NET Framework Data Provider for SQL Server `{1634cdd7-0888-42e3-9fa2-b6d32563b91d}`
pub const DATA_PROVIDER_FOR_SQL_SERVER: Uuid = uuid!("1634cdd7-0888-42e3-9fa2-b6d32563b91d");

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
