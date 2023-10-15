use std::io::stdout;
use std::io::Write;

use crate::geo_struct::{ReaderElement, UniformArrayType};

pub fn preview(elem: &ReaderElement) {
    to_json(elem, &mut stdout());
}

pub fn to_json(elem: &ReaderElement, output: &mut dyn Write) {
    write_element(output, elem, 0);
}

const ERRMSG: &str = "write error!";

fn write_tabs(output: &mut dyn Write, tabs: usize) {
    for _ in 0..tabs {
        write!(output, "  ").expect(ERRMSG);
    }
}

enum WroteWhat {
    Init,
    WroteInline,
    WroteBlock,
}

macro_rules! uniform_array_loop_print {
    ($output:ident, $vec:ident) => {
        write!($output, "{}", $vec[0]).expect(ERRMSG);
        for el in &$vec[1..] {
            write!($output, ", {}", el).expect(ERRMSG);
        }
    };
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
            let mut wrote_in_line = 0;
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
                        wrote_in_line = 0;
                    }
                    _ => {
                        if let WroteWhat::WroteBlock = last_wrote_what {
                            writeln!(output, "").expect(ERRMSG);
                            write_tabs(output, tabs + 1);
                        }
                        if wrote_in_line >= 20 {
                            wrote_in_line = 0;
                            writeln!(output, "").expect(ERRMSG);
                            write_tabs(output, tabs + 1);
                        }
                        write_element(output, elem, 0);
                        if i != arr_last_i {
                            write!(output, ", ").expect(ERRMSG);
                        }
                        wrote_in_line += 1;

                        last_wrote_what = WroteWhat::WroteInline;
                    }
                }
            }
            writeln!(output, "").expect(ERRMSG);
            write_tabs(output, tabs);
            write!(output, "]").expect(ERRMSG);
        }
        ReaderElement::UniformArray(uniarr) => {
            write!(output, "[").expect(ERRMSG);
            match uniarr {
                // TODO: can this be done with a macro?
                UniformArrayType::UniformArrayTbool(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                UniformArrayType::UniformArrayTu8(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                UniformArrayType::UniformArrayTu16(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                // UniformArrayType::UniformArrayTu32(vec) if vec.len() > 0 => {
                //     uniform_array_loop_print!(output, vec);
                // }
                // UniformArrayType::UniformArrayTu64(vec) if vec.len() > 0 => {
                //     uniform_array_loop_print!(output, vec);
                // }
                UniformArrayType::UniformArrayTi8(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                UniformArrayType::UniformArrayTi16(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                UniformArrayType::UniformArrayTi32(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                UniformArrayType::UniformArrayTi64(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                UniformArrayType::UniformArrayTf16(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                UniformArrayType::UniformArrayTf32(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                UniformArrayType::UniformArrayTf64(vec) if vec.len() > 0 => {
                    uniform_array_loop_print!(output, vec);
                }
                &_ => { panic!("wtf is this?"); }
            };
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
