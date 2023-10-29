use anyhow::Context;
use mapr::Mmap;
use std::io::Cursor;
use std::path::PathBuf;
use std::{fs::File, time::UNIX_EPOCH};
use sysdiagram::{get_settings, LoadError, SysDiagramFile};

#[derive(argh::FromArgs)]
/// parse a sysdiagram from a FDB file
struct Options {
    /// path to the FDB file
    #[argh(positional)]
    file: PathBuf,

    #[argh(switch)]
    /// assume the file is base64 encoded
    base64: bool,

    #[argh(switch)]
    /// print relationships
    relationships: bool,

    #[argh(switch)]
    /// print clsid table
    classes: bool,

    #[argh(switch)]
    /// print cfb streams
    streams: bool,

    #[argh(switch)]
    /// print settings
    settings: bool,

    #[argh(switch)]
    /// print dsref
    dsref: bool,

    #[argh(switch)]
    /// print tables
    tables: bool,

    #[argh(switch)]
    /// print label
    labels: bool,
}

fn load_database(opts: &Options) -> Result<(), anyhow::Error> {
    // Load the database file
    let file = File::open(&opts.file)
        .with_context(|| format!("Failed to open input file '{}'", opts.file.display()))?;
    let mmap = unsafe { Mmap::map(&file)? };
    let buf: &[u8] = &mmap;
    let cursor = Cursor::new(buf);

    if opts.base64 {
        unimplemented!("--base64 is unimplemented");
    }

    let mut reader = SysDiagramFile::open(cursor).map_err(LoadError::Cfb)?;

    if opts.streams {
        let root = reader.root_entry();
        let ctime = root.created().duration_since(UNIX_EPOCH);
        let mtime = root.modified().duration_since(UNIX_EPOCH);
        eprintln!("created: {:?}, modified: {:?}", ctime, mtime);

        eprintln!("Root CLSID: {}", root.clsid());
        let entries = reader.read_root_storage();

        eprintln!("CFB Streams:");
        for entry in entries {
            println!("- {:?}: {}", entry.name(), entry.path().display());
        }
    }

    let comp_obj = reader.root_comp_obj()?;
    println!("{:?}", comp_obj);

    eprintln!("Parsing DSREF-SCHEMA-CONTENT");
    let dsref_schema_contents = reader.dsref_schema_contents()?;
    if opts.settings {
        if let Ok(settings) = get_settings(dsref_schema_contents.root_node.name.as_ref().unwrap()) {
            for (key, value) in &settings {
                println!("{:25}: {}", key, value);
            }
        } else {
            eprintln!(
                "Failed to parse connection string:\n{:?}",
                dsref_schema_contents.root_node.name
            );
        }
    }
    if opts.dsref {
        eprintln!("{:#?}", dsref_schema_contents);
    }

    let (form_control, tables, relationships, labels) = reader.schema_form()?;
    if opts.classes {
        for c in form_control.site_classes {
            eprintln!("- {:?}", c);
        }
    }

    if opts.labels {
        for label in labels {
            println!("{:?}", label);
        }
    }

    if opts.tables {
        for table in tables {
            println!("{}.{}", table.sch_grid.schema, table.sch_grid.name);
            eprintln!("{:?}", table.sch_grid);
        }
    }
    if opts.relationships {
        for relationship in relationships {
            println!(
                "{:60} {:25} {:25}",
                relationship.name, relationship.from, relationship.to
            );
            println!("{:?}", relationship.control);
        }
    }
    Ok(())
}

pub fn main() -> Result<(), anyhow::Error> {
    let opts: Options = argh::from_env();
    load_database(&opts).with_context(|| "Loading database failed!")
}
