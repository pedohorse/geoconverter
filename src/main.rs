use geoconverter::{
    create_stl_solid, parse_ascii, parse_binary, serialize_obj, serialize_stl, HoudiniGeoSchemaParser, ReaderElement,
};
use std::env::args;
use std::fs::File;
use std::io::{self, Write};

enum ConvertionType {
    Obj,
    Stl,
}

fn main() {
    let mut argv = args();
    if argv.len() < 2 {
        println!(
            "usage: geoconverter [-t type] output_file

    -t type (default=obj)
    "
        );
        std::process::exit(2)
    }
    let mut stdin = io::stdin();

    let res = parse_ascii(&mut stdin);

    let arg1 = argv.nth(1).expect("not enough arguments provided!");
    let (converion_type, out_file_path) = if arg1 == "-t" {
        let arg2 = argv.next();
        let file_path = argv.next().unwrap_or_else(|| {
            println!("output path not provided!");
            std::process::exit(1);
        });
        match arg2.as_deref() {
            Some("obj") => (ConvertionType::Obj, file_path),
            Some("stl") => (ConvertionType::Stl, file_path),
            Some(s) => {
                println!("wtf is type {}?", s);
                std::process::exit(1);
            }
            _ => {
                println!("type not provided");
                std::process::exit(1);
            }
        }
    } else {
        (ConvertionType::Obj, arg1)
    };

    match converion_type {
        ConvertionType::Obj => convert_to_obj(&res, &out_file_path),
        ConvertionType::Stl => convert_to_stl(&res, &out_file_path),
    }
}

fn convert_to_stl(res: &ReaderElement, path: &str) {
    let stlsolid = create_stl_solid(&mut HoudiniGeoSchemaParser::new(res));

    let mut file = io::BufWriter::new(File::create(path).expect("could not create output file"));

    serialize_stl(&stlsolid, &mut file);
    file.flush().expect("failed to flush the file");
}

fn convert_to_obj(res: &ReaderElement, path: &str) {
    let mut schema_parser = HoudiniGeoSchemaParser::new(res);
    let mut file = io::BufWriter::new(File::create(path).expect("could not create output file"));

    serialize_obj(&mut schema_parser, &mut file);
    file.flush().expect("failed to flush the file");
}
