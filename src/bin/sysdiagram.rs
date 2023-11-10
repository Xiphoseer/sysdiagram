use anyhow::Context;
use mapr::Mmap;
use ms_oforms::controls::user_form::FormControl;
use ms_oforms::properties::color::{OleColor, RgbColor};
use ms_oforms::properties::{Position, Size};
use std::io::Cursor;
use std::path::PathBuf;
use std::{fs::File, time::UNIX_EPOCH};
use sysdiagram::dds::DdsPolylineEndType;
use sysdiagram::dsref::DSRefSchemaContents;
use sysdiagram::{get_settings, Control, Error, SiteInfo, SysDiagramFile};

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
    /// print form
    form: bool,

    #[argh(switch)]
    /// print form size
    size: bool,

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

    #[argh(switch)]
    /// print \0CompObj info
    comp_obj: bool,

    #[argh(switch)]
    /// generate SVG
    svg: bool,

    #[argh(switch)]
    /// enable SVG visual debug nodes
    debug: bool,
}

fn color(r: OleColor) -> RgbColor {
    match r {
        OleColor::Default(d) | OleColor::RgbColor(d) => d,
        OleColor::SystemPalette(p) => {
            let color = p
                .as_system_color()
                .expect("expected well-known system palette index");
            RgbColor::from(color)
        }
        OleColor::PaletteEntry(e) => todo!("{:?}", e),
    }
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

    let mut reader = SysDiagramFile::open(cursor).map_err(Error::Cfb)?;

    if opts.streams && !opts.svg {
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
    if opts.comp_obj && !opts.svg {
        println!("{:?}", comp_obj);
    }

    eprintln!("Parsing DSREF-SCHEMA-CONTENT");
    let dsref_schema_contents = reader.dsref_schema_contents()?;
    if opts.settings && !opts.svg {
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
    if opts.dsref && !opts.svg {
        println!("time: {}", dsref_schema_contents.get_time());
        println!("{:#?}", dsref_schema_contents);
    }

    let (form_control, controls) = reader.schema_form()?;

    if opts.svg {
        generate_svg(&dsref_schema_contents, &controls, &form_control, opts.debug);
        return Ok(());
    }

    if opts.form {
        println!("{:#?}", form_control);
    }
    if opts.size {
        println!("logical: {:?}", form_control.logical_size);
        println!("displayed: {:?}", form_control.displayed_size);
        println!("scroll: {:?}", form_control.scroll_position);
    }
    if opts.classes {
        for c in form_control.site_classes {
            println!("- {:?}", c);
        }
    }

    for (site, control) in controls.iter().filter(|(_, c)| match c {
        Control::Label(_) => opts.labels,
        Control::Polyline(_) => opts.relationships,
        Control::SchGrid(_) => opts.tables,
        _ => false,
    }) {
        println!("==> {:?}", site);
        match control {
            Control::SchGrid(sch_grid) => {
                println!("{:?}", sch_grid.extent);
                println!("{:?}", sch_grid.frame);
                println!("{:?}", sch_grid.data_source);
            }
            Control::Label(label) => {
                println!("{:?}", label);
            }
            Control::Polyline(polyline) => {
                println!("{:?}", polyline);
            }
            Control::Unknown(_clsid) => {
                // TODO?
            }
        }
    }
    Ok(())
}

fn generate_svg(
    dsref_schema_contents: &DSRefSchemaContents,
    controls: &[(SiteInfo, Control)],
    form_control: &FormControl,
    debug: bool,
) {
    let title = dsref_schema_contents.root_node.children[0]
        .name
        .as_deref()
        .unwrap();
    println!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    println!(r#"<svg xmlns="http://www.w3.org/2000/svg""#);
    println!(r#"    xmlns:xlink="http://www.w3.org/1999/xlink""#);
    println!(r#"    version="1.1" baseProfile="full""#);
    let min_x = controls.iter().map(|(s, _)| s.pos.left).min().unwrap() as f32 / 100.0;
    let min_y = controls.iter().map(|(s, _)| s.pos.top).min().unwrap() as f32 / 100.0;
    let (f_width, f_height) = size_himetric_to_mm(form_control.logical_size);
    println!(r#"    width="{}mm" height="{}mm""#, f_width, f_height);
    println!(
        r#"    viewBox="{} {} {} {}""#,
        min_x - 10.0,
        min_y - 10.0,
        f_width,
        f_height
    );
    println!(
        r#"    style="background-color: {}""#,
        color(form_control.back_color)
    );
    println!(">");
    println!(r#"    <title>{}</title>"#, title);
    println!(r#"    <desc>Beschreibung/Textalternative zum Inhalt.</desc>"#);
    println!(r#"<circle cx="0" cy="0" r="4" fill="red" />"#);
    for (site, control) in controls {
        let (x, y) = pos_himetric_to_mm(&site.pos);
        match control {
            Control::SchGrid(sch_grid) => {
                if debug {
                    println!(r#"<circle cx="{}" cy="{}" r="2" fill="blue" />"#, x, y);
                }
                let (w, h) = size_himetric_to_mm(sch_grid.extent);
                println!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" stroke="{}" stroke-width="1" fill="none" />"#,
                    x, y, w, h, "red"
                );
                /*if debug {
                    let (w2, h2) = size_himetric_to_mm(sch_grid.frame._d5_2);
                    println!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" stroke="{}" stroke-width="0.5" fill="none" />"#,
                        x, y, w2, h2, "purple"
                    );
                }*/

                println!(
                    r#"<text x="{}" y="{}" font-size="4" font-family="Tahoma">{}</text>"#,
                    x + 2.0,
                    y + 6.0,
                    sch_grid.frame.name
                );
            }
            Control::Label(label) => {
                if debug {
                    println!(r#"<circle cx="{}" cy="{}" r="2" fill="red" />"#, x, y);
                }
                let (width, height) = size_himetric_to_mm(label.size);
                let bg_rgb = color(label.back_color);
                let fg_rgb = color(label.fore_color);
                println!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" />"#,
                    x, y, width, height, bg_rgb,
                );
                println!(
                    r#"<text font-family="{}" color="{}" font-size="{}" id="c{}" x="{}" y="{}">{}</text>"#,
                    label.font.font_face,
                    fg_rgb,
                    8.25 * 0.35,
                    site.id,
                    x,
                    y + height * 0.8,
                    label.text
                );
            }
            Control::Polyline(line) => {
                if debug {
                    println!(r#"<circle cx="{}" cy="{}" r="2" fill="green" />"#, x, y);
                    let (lx, ly) = pos_himetric_to_mm(&line.label_pos);
                    println!(r#"<circle cx="{}" cy="{}" r="4" fill="cyan" />"#, lx, ly);
                }
                print!(
                    r#"<polyline stroke-width="1" id="c{}" fill="none" stroke="{}" points=""#,
                    site.id,
                    color(line.color),
                );
                fn cap_color(cap: DdsPolylineEndType) -> &'static str {
                    match cap {
                        DdsPolylineEndType::Many => "yellow",
                        DdsPolylineEndType::Key => "orange",
                        _ => "black",
                    }
                }
                for p in &line.positions {
                    let (x, y) = pos_himetric_to_mm(p);
                    print!("{},{} ", x, y);
                }
                println!("\" />");
                let color_src = cap_color(line.end_type_src);
                let color_dest = cap_color(line.end_type_dest);

                let (x_src, y_src) = pos_himetric_to_mm(line.positions.first().unwrap());
                let (x_dest, y_dest) = pos_himetric_to_mm(line.positions.last().unwrap());
                print!(
                    r#"<circle cx="{}" cy="{}" r="2" fill="{}" />"#,
                    x_src, y_src, color_src
                );
                print!(
                    r#"<circle cx="{}" cy="{}" r="2" fill="{}" />"#,
                    x_dest, y_dest, color_dest
                );
            }
            Control::Unknown(_) => {}
        }
    }
    println!("</svg>");
}

fn pos_himetric_to_mm(p: &Position) -> (f32, f32) {
    (p.left as f32 / 100.0, p.top as f32 / 100.0)
}

fn size_himetric_to_mm(size: Size) -> (f32, f32) {
    (size.width as f32 / 100.0, size.height as f32 / 100.0)
}

pub fn main() -> Result<(), anyhow::Error> {
    let opts: Options = argh::from_env();
    load_database(&opts).with_context(|| "Loading database failed!")
}
