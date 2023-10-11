use std::io::Write;
use std::io::stdout;

use crate::geo_struct::ReaderElement;

pub fn preview(elem: &ReaderElement) {
    to_json(elem, &mut stdout());
}


pub fn to_json(elem: &ReaderElement, output: &mut dyn Write) {
    write_element(output, elem, 0);
}

const ERRMSG: &str = "write error!";

fn write_tabs(output: &mut dyn Write, tabs: usize) {
    for _ in 0..tabs { write!(output, "  ").expect(ERRMSG); }
}

enum WroteWhat {
    Init,
    WroteInline,
    WroteBlock
}

fn write_element(output: &mut dyn Write, elem: &ReaderElement, tabs: usize) {
    match elem {        
        ReaderElement::Array(x) if x.len() == 0 => {
            write!(output, "[]").expect(ERRMSG);
        }
        ReaderElement::Array(x) => {
            writeln!(output, "[").expect(ERRMSG);
            write_tabs(output, tabs + 1);
            let arr_last_i = x.len() - 1;
            let mut last_wrote_what = WroteWhat::Init;
            for (i, elem) in x.iter().enumerate() {
                match elem {
                    ReaderElement::Array(_) | ReaderElement::KeyValueObject(_) => {
                        if let WroteWhat::WroteInline | WroteWhat::WroteBlock = last_wrote_what {
                            writeln!(output, "").expect(ERRMSG);
                            write_tabs(output, tabs + 1);
                        }
                        write_element(output, elem, tabs + 1);
                        if i != arr_last_i {
                            write!(output, ", ").expect(ERRMSG);
                        }
                        write_tabs(output, tabs + 1);
                        last_wrote_what = WroteWhat::WroteBlock;
                    }
                    _ => {
                        if let WroteWhat::WroteBlock = last_wrote_what {
                            writeln!(output, "").expect(ERRMSG);
                            write_tabs(output, tabs + 1);
                        }
                        write_element(output, elem, 0);
                        if i != arr_last_i {
                            write!(output, ", ").expect(ERRMSG);
                        }
                        last_wrote_what = WroteWhat::WroteInline;
                    }
                }
            }
            writeln!(output, "").expect(ERRMSG);
            write_tabs(output, tabs);
            write!(output, "]").expect(ERRMSG);
        }
        ReaderElement::KeyValueObject(x) if x.len() == 0 => {
            write!(output, "{{}}").expect(ERRMSG);
        }
        ReaderElement::KeyValueObject(x) => {
            write!(output, "{{").expect(ERRMSG);
            let arr_last_i = x.len() - 1;
            for (i, (key, elem)) in x.iter().enumerate() {
                writeln!(output, "").expect(ERRMSG);
                write_tabs(output, tabs + 1);
                match elem {
                    ReaderElement::Array(_) | ReaderElement::KeyValueObject(_) => {
                        writeln!(output, "\"{}\":", key).expect(ERRMSG);
                        write_tabs(output, tabs + 1);
                        write_element(output, elem, tabs + 1);
                        if i != arr_last_i {
                            write!(output, ", ").expect(ERRMSG);
                        }
                        write_tabs(output, tabs + 1);
                    }
                    _ => {
                        write!(output, "\"{}\": ", key).expect(ERRMSG);
                        write_element(output, elem, 0);
                        if i != arr_last_i {
                            write!(output, ", ").expect(ERRMSG);
                        }
                    }
                }
            }
            writeln!(output, "").expect(ERRMSG);
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