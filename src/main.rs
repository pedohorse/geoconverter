use geoconverter::{create_stl_solid, parse, serialize_obj, serialize_stl, HoudiniGeoSchemaParser, ReaderElement};
use std::env::args;
use std::fs::File;
use std::io::{self, Write};

enum ConvertionType {
    Obj,
    Stl,
    Geo,
}

enum InputType {
    Stdin(io::StdinLock<'static>),
    File(io::BufReader<File>),
}

enum OutputType {
    Stdout(io::StdoutLock<'static>),
    File(io::BufWriter<File>),
}

const HELP_MESSAGE: &str = "
usage: geoconverter [-t type] [input_file] [output_file]
    
    -t type (default=obj)

If last 2 arguments are file paths - 
  first is interpreted as input file path,
  second is interpreted as output file path
If just ONE file path provided - it's interpreted as output file path
  input is taken from stdin
If NO file paths provided -
  input is taken from stdin
  output is written to stdout
";

fn main() {
    let mut argv = args();
    // TODO: this argument parsing started out fine, but now got bloated and confusing.
    // If a single more arg is added - refactor this all shit
    // Do a struct with input parsing options, let one arg parsing function return it

    let arg1 = argv.nth(1);

    let (converion_type, file_path) = match arg1 {
        Some(t) if t == "--help" => {
            // print usage and exit
            println!("{}", HELP_MESSAGE);
            std::process::exit(2)
        }
        Some(t) if t == "-t" => {
            let arg2 = argv.next();
            let file_path = argv.next().unwrap_or_else(|| "".to_owned());
            match arg2.as_deref() {
                Some("obj") => (ConvertionType::Obj, file_path),
                Some("stl") => (ConvertionType::Stl, file_path),
                Some("geo") | Some("json") => (ConvertionType::Geo, file_path),
                Some(s) => {
                    println!("wtf is type {}?", s);
                    std::process::exit(1);
                }
                _ => {
                    println!("type not provided");
                    std::process::exit(1);
                }
            }
        }
        Some(s) => (ConvertionType::Obj, s),
        _ => (ConvertionType::Obj, "".to_owned()),
    };

    let (mut in_file, mut out_file) = if let Some(another_file_path) = argv.next() {
        (
            InputType::File(io::BufReader::new(File::open(file_path).expect("failed to open input file"))),
            OutputType::File(io::BufWriter::new(
                File::create(another_file_path).expect("could not create output file"),
            )),
        )
    } else {
        (
            InputType::Stdin(io::stdin().lock()),
            if file_path == "" {
                OutputType::Stdout(io::stdout().lock())
            } else {
                OutputType::File(io::BufWriter::new(
                    File::create(file_path).expect("could not create output file"),
                ))
            },
        )
    };

    // input parsing
    let res = parse(match in_file {
        InputType::File(ref mut x) => x,
        InputType::Stdin(ref mut x) => x,
    });

    // output
    let out_ref: &mut dyn Write = match out_file {
        OutputType::Stdout(ref mut f) => f,
        OutputType::File(ref mut f) => f,
    };

    match converion_type {
        ConvertionType::Obj => convert_to_obj(&res, out_ref),
        ConvertionType::Stl => convert_to_stl(&res, out_ref),
        ConvertionType::Geo => convert_to_geo(&res, out_ref),
    }

    // don't forget to flush (but does it matter in the end of the program?)
    match out_file {
        OutputType::File(ref mut file) => file.flush().expect("failed to flush the file"),
        OutputType::Stdout(ref mut file) => file.flush().expect("failed to flush stdout"),
    }
}

fn convert_to_stl(res: &ReaderElement, out: &mut dyn io::Write) {
    let stlsolid = create_stl_solid(&mut HoudiniGeoSchemaParser::new(res));

    serialize_stl(&stlsolid, out);
}

fn convert_to_obj(res: &ReaderElement, out: &mut dyn io::Write) {
    let mut schema_parser = HoudiniGeoSchemaParser::new(res);

    serialize_obj(&mut schema_parser, out);
}

fn convert_to_geo(res: &ReaderElement, out: &mut dyn io::Write) {
    geoconverter::geo_struct_serializer::to_json(res, out);
}
