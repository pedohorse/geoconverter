use geoconverter::{create_stl_solid, parse, serialize_obj, serialize_stl, HoudiniGeoSchemaParser, ReaderElement};
use std::env::{args, Args};
use std::fs::File;
use std::io::{self, Write};

enum ConvertionType {
    Obj,
    Stl,
    Geo,
    Bgeo,
}

enum InputType {
    Stdin(io::StdinLock<'static>),
    File(io::BufReader<File>),
}

enum OutputType {
    Stdout(io::StdoutLock<'static>),
    File(io::BufWriter<File>),
}

struct ArgumentOptions {
    convertion_type: ConvertionType,
    input_type: InputType,
    output_type: OutputType,
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
    argv.next().expect("zero argument not provided? unexpected");

    let mut options = match parse_arguments(&mut argv) {
        Ok(x) => x,
        Err(e) => {
            println!("Error parsing arguments: {}\n\n", e.ohnoo);
            print!("{}", HELP_MESSAGE);
            std::process::exit(2);
        }
    };

    // input parsing
    let res = parse(match options.input_type {
        InputType::File(ref mut x) => x,
        InputType::Stdin(ref mut x) => x,
    });

    // output
    let out_ref: &mut dyn Write = match options.output_type {
        OutputType::Stdout(ref mut f) => f,
        OutputType::File(ref mut f) => f,
    };

    // convertion
    match options.convertion_type {
        ConvertionType::Obj => convert_to_obj(&res, out_ref),
        ConvertionType::Stl => convert_to_stl(&res, out_ref),
        ConvertionType::Geo => geoconverter::geo_struct_serializer::to_json(&res, out_ref),
        ConvertionType::Bgeo => geoconverter::bgeo_struct_serializer::to_bjson(&res, out_ref),
    }

    // don't forget to flush (but does it matter in the end of the program?)
    match options.output_type {
        OutputType::File(ref mut file) => file.flush().expect("failed to flush the file"),
        OutputType::Stdout(ref mut file) => file.flush().expect("failed to flush stdout"),
    }
}

enum ExpectedFlag {
    NotExpecting,
    ExpectingType,
}

struct ArgumentParsingError {
    ohnoo: String,
}

fn parse_arguments(argv: &mut dyn Iterator<Item = String>) -> Result<ArgumentOptions, ArgumentParsingError> {
    let mut convertion_type = ConvertionType::Obj;
    let mut input_type: Option<InputType> = None;
    let mut output_type: Option<OutputType> = None;
    let mut flags = ExpectedFlag::NotExpecting;
    let mut stashed_path: Option<String> = None;

    for arg in argv {
        match (arg.as_str(), &flags) {
            ("-t", ExpectedFlag::NotExpecting) => {
                flags = ExpectedFlag::ExpectingType;
            }
            (t, ExpectedFlag::ExpectingType) => {
                flags = ExpectedFlag::NotExpecting;
                convertion_type = match t {
                    "obj" => ConvertionType::Obj,
                    "stl" => ConvertionType::Stl,
                    "geo" | "json" => ConvertionType::Geo,
                    "bgeo" => ConvertionType::Bgeo,
                    s => {
                        println!("wtf is type {}?", s);
                        return Err(ArgumentParsingError {
                            ohnoo: format!("unknown type '{}'", s),
                        });
                    }
                }
            }
            (file_path, ExpectedFlag::NotExpecting) => {
                match &stashed_path {
                    None => {
                        stashed_path = Some(arg);
                    }
                    Some(input_path) => {  // else it's the second positional argument, so we are ready to assign
                        input_type = Some(InputType::File(io::BufReader::new(
                            File::open(input_path).expect("failed to open input file"),
                        )));
                        output_type = Some(OutputType::File(io::BufWriter::new(
                            File::create(file_path).expect("could not create output file"),
                        )));
                    }
                }
            }
        }
    }
    if let None = output_type {
        if let Some(file_path) = stashed_path {
            output_type = Some(OutputType::File(io::BufWriter::new(
                File::create(file_path).expect("could not create output file")
            )));
        } else {
            output_type = Some(OutputType::Stdout(io::stdout().lock()));
        }
    }
    if let None = input_type {
        input_type = Some(InputType::Stdin(io::stdin().lock()));
    };

    Ok(ArgumentOptions {
        convertion_type: convertion_type,
        input_type: input_type.expect("impossible!"),
        output_type: output_type.expect("impossible!"),
    })
}

fn convert_to_stl(res: &ReaderElement, out: &mut dyn io::Write) {
    let stlsolid = create_stl_solid(&mut HoudiniGeoSchemaParser::new(res));

    serialize_stl(&stlsolid, out);
}

fn convert_to_obj(res: &ReaderElement, out: &mut dyn io::Write) {
    let mut schema_parser = HoudiniGeoSchemaParser::new(res);

    serialize_obj(&mut schema_parser, out);
}

///
/// --------------------------------------------------------------
///                            TESTS
/// --------------------------------------------------------------
///

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    struct TempFile {
        path: &'static str,
    }

    impl TempFile {
        fn new(path: &'static str) -> TempFile {
            let mut file = File::create(path).expect("failed to create test file");
            let n = file.write(path.as_bytes()).expect("failed to write to temp file");
            TempFile { path: path }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            std::fs::remove_file(self.path).expect("failed to remove temporary file!");
        }
    }

    #[test]
    fn argparser_simple() {
        // check1
        match parse_arguments(&mut vec![].into_iter()) {
            Ok(ArgumentOptions {
                convertion_type: ConvertionType::Obj,
                input_type: InputType::Stdin(_),
                output_type: OutputType::Stdout(_),
            }) => {
                println!("check1 succ!");
            }
            _ => {
                assert!(false, "argument parsing failed");
            }
        }

        let foo_in = TempFile::new("temp_foo_in");
        let foo_out = TempFile::new("temp_foo_out");

        // check2
        match parse_arguments(&mut vec![foo_in.path.to_owned(), foo_out.path.to_owned()].into_iter()) {
            Ok(ArgumentOptions {
                convertion_type: ConvertionType::Obj,
                input_type: InputType::File(mut fi),
                output_type: OutputType::File(mut fo),
            }) => {
                let mut buf = Vec::new();
                fi.read_to_end(&mut buf).expect("failed to read from test input file");
                assert_eq!(foo_in.path.as_bytes(), buf);
                fo.write_all(&buf).expect("failed to write to test output file");
                // TODO: test output file's contents

                println!("check2 succ!");
            }
            _ => {
                assert!(false, "argument parsing failed");
            }
        }

        // check3
        match parse_arguments(&mut vec![foo_out.path.to_owned()].into_iter()) {
            Ok(ArgumentOptions {
                convertion_type: ConvertionType::Obj,
                input_type: InputType::Stdin(_),
                output_type: OutputType::File(_),
            }) => {
                println!("check3 succ!");
            }
            _ => {
                assert!(false, "argument parsing failed");
            }
        }

        // check4
        match parse_arguments(&mut vec!["-t".to_owned(), "bgeo".to_owned(), foo_out.path.to_owned()].into_iter()) {
            Ok(ArgumentOptions {
                convertion_type: ConvertionType::Bgeo,
                input_type: InputType::Stdin(_),
                output_type: OutputType::File(_),
            }) => {
                println!("check4 succ!");
            }
            _ => {
                assert!(false, "argument parsing failed");
            }
        }

        // check5
        match parse_arguments(&mut vec![foo_out.path.to_owned(), "-t".to_owned(), "geo".to_owned()].into_iter()) {
            Ok(ArgumentOptions {
                convertion_type: ConvertionType::Geo,
                input_type: InputType::Stdin(_),
                output_type: OutputType::File(_),
            }) => {
                println!("check5 succ!");
            }
            _ => {
                assert!(false, "argument parsing failed");
            }
        }

        // check6
        match parse_arguments(
            &mut vec![
                foo_in.path.to_owned(),
                "-t".to_owned(),
                "geo".to_owned(),
                foo_out.path.to_owned(),
            ]
            .into_iter(),
        ) {
            Ok(ArgumentOptions {
                convertion_type: ConvertionType::Geo,
                input_type: InputType::File(_),
                output_type: OutputType::File(_),
            }) => {
                println!("check6 succ!");
            }
            _ => {
                assert!(false, "argument parsing failed");
            }
        }
    }
}
