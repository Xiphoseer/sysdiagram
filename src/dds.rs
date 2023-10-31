//! # DaVinci Design Surface (DDS)
//!
//! From: <https://social.msdn.microsoft.com/Forums/en-US/1d88df38-23d4-48a6-ab77-3b99dc1e9a24/why-binary-annotations?forum=sqlintegrationservices>
//! > DDS (DaVinci Design Surface) is an internal name for the design surface we used for our designers.
//! > It is originally developed by VS and I think it was also used for a few other designers too
//! > (like the UI designers in VS 6). It is a pretty old design surface and it will soon be replaced everywhere it was used.
//!
//! See also:
//! - <https://fjehl.wordpress.com/2010/05/23/ssis-modifier-lapparence-des-packages-via-lapi-fail/>
//! - <http://sqlsoundings.blogspot.com/2011/10/ssis-word-wrapping-annotations-using.html>
//! - <https://www.sqlservercentral.com/articles/hidden-ssis-features-word-wrapping-your-annotations-and-more>

use uuid::{uuid, Uuid};

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
