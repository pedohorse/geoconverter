use std::io::Write;
use std::io::stdout;

use crate::geo_struct::ReaderElement;

pub fn preview(elem: &ReaderElement) {
    to_json(elem, &mut stdout());
}


pub fn to_json(elem: &ReaderElement, output: &mut dyn Write) {
    write_offset(output, elem, 0);
}

const ERRMSG: &str = "write error!";

fn write_tabs(output: &mut dyn Write, tabs: usize) {
    for _ in 0..tabs { write!(output, "  ").expect(ERRMSG); }
}


fn write_offset(output: &mut dyn Write, elem: &ReaderElement, tabs: usize) {
    match elem {        
        ReaderElement::Array(x) => {
            writeln!(output, "[").expect(ERRMSG);
            write_tabs(output, tabs + 1);
            for elem in x {
                match elem {
                    ReaderElement::Array(_) | ReaderElement::KeyValueObject(_) => {
                        writeln!(output, "").expect(ERRMSG);
                        write_tabs(output, tabs + 1);
                        write_offset(output, elem, tabs + 1);
                        write_tabs(output, tabs + 1);
                    }
                    _ => {
                        write_offset(output, elem, 0);
                        write!(output, ", ").expect(ERRMSG);
                    }
                }
            }
            writeln!(output, "").expect(ERRMSG);
            write_tabs(output, tabs);
            write!(output, "]").expect(ERRMSG);
        }
        ReaderElement::KeyValueObject(x) => {
            writeln!(output, "{{").expect(ERRMSG);
            for (key, elem) in x {
                write_tabs(output, tabs + 1);
                match elem {
                    ReaderElement::Array(_) | ReaderElement::KeyValueObject(_) => {
                        writeln!(output, "{}:", key).expect(ERRMSG);
                        write_tabs(output, tabs + 1);
                        write_offset(output, elem, tabs + 1);
                        write_tabs(output, tabs + 1);
                    }
                    _ => {
                        write!(output, "{}: ", key).expect(ERRMSG);
                        write_offset(output, elem, 0);
                        write!(output, "").expect(ERRMSG);
                    }
                }
                writeln!(output, "").expect(ERRMSG);
                write_tabs(output, tabs);
                writeln!(output, "").expect(ERRMSG);
            }
            write_tabs(output, tabs);
            write!(output, "}}").expect(ERRMSG);
        }
        ReaderElement::Bool(x) => {
            write!(output, "{}", x).expect(ERRMSG);
        }
        ReaderElement::Int(x) => {
            write!(output, "{}", x).expect(ERRMSG);
        }
        ReaderElement::Float(x) => {
            write!(output, "{}", x).expect(ERRMSG);
        }
        ReaderElement::None => {
            write!(output, "None").expect(ERRMSG);
        }
        ReaderElement::Text(x) => {
            write!(output, "\"{}\"", x).expect(ERRMSG);
        }
    };
}