use crate::geo_struct::ReaderElement;

pub fn preview(elem: &ReaderElement) {
    print_offset(elem, 0);
}

fn print_tabs(tabs: usize) {
    for _ in 0..tabs { print!("\t"); }
}

fn print_offset(elem: &ReaderElement, tabs: usize) {
    match elem {        
        ReaderElement::Array(x) => {
            println!("[");
            print_tabs(tabs + 1);
            for elem in x {
                match elem {
                    ReaderElement::Array(_) | ReaderElement::KeyValueObject(_) => {
                        print_offset(elem, tabs + 1);
                        print_tabs(tabs + 1);
                    }
                    _ => {
                        print_offset(elem, 0);
                        print!(", ")
                    }
                }
            }
            println!("");
            print_tabs(tabs);
            println!("]");
        }
        ReaderElement::KeyValueObject(x) => {
            println!("{{");
            for (key, elem) in x {
                print_tabs(tabs + 1);
                match elem {
                    ReaderElement::Array(_) | ReaderElement::KeyValueObject(_) => {
                        println!("{}:", key);
                        print_tabs(tabs + 1);
                        print_offset(elem, tabs + 1);
                        print_tabs(tabs + 1);
                    }
                    _ => {
                        print!("{}: ", key);
                        print_offset(elem, 0);
                        println!("");
                    }
                }
                
                println!("");
            }
            print_tabs(tabs);
            println!("}}");
        }
        ReaderElement::Bool(x) => {
            print!("{}", x);
        }
        ReaderElement::Int(x) => {
            print!("{}", x);
        }
        ReaderElement::Float(x) => {
            print!("{}", x);
        }
        ReaderElement::None => {
            print!("None");
        }
        ReaderElement::Text(x) => {
            print!("{}", x);
        }
    };
}