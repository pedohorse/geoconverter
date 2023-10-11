use geoconverter::{self, ReaderElement};

use std::fs::File;
use std::io::{BufReader, Read};

#[test]
fn parse_bgeo_box() {
    parse_box_helper("./tests/box.bgeo", &geoconverter::parse_binary);
}

#[test]
fn parse_geo_box() {
    parse_box_helper("./tests/box.geo", &geoconverter::parse_ascii);
}

#[test]
fn parse_bgeo_box_autodetect() {
    parse_box_helper("./tests/box.bgeo", &geoconverter::parse);
}

#[test]
fn parse_geo_box_autodetect() {
    parse_box_helper("./tests/box.geo", &geoconverter::parse);
}

fn parse_box_helper(filepath: &str, parser: & dyn Fn(&mut dyn Read) -> ReaderElement) {
    let mut f = File::open(filepath).expect("failed to open test file");
    //let mut f = File::open("/tmp/filea.bgeo").expect("failed to open test file");
    let elem = parser(&mut BufReader::new(&mut f));

    geoconverter::geo_struct_serializer::preview(&elem);

    if let ReaderElement::Array(root_arr) = &elem {
        if let ReaderElement::Text(x) = &root_arr[0] {
            assert_eq!(x, "fileversion");
        } else {
            assert!(false);
        }
        if let ReaderElement::Array(topo_arr) = &root_arr[13] {
            if let ReaderElement::Array(pref_arr) = &topo_arr[1] {
                if let ReaderElement::Array(ind_arr) = &pref_arr[1] {
                    let expected_indices = [
                        0_i64, 1, 3, 2, 4, 5, 7, 6, 6, 7, 2, 3, 5, 4, 1, 0, 5, 0, 2, 7, 1, 4, 6, 3,
                    ];
                    assert_eq!(ind_arr.len(), expected_indices.len());
                    for (i, idx) in expected_indices.iter().enumerate() {
                        if let ReaderElement::Int(x) = ind_arr[i] {
                            assert_eq!(x, *idx);
                        } else {
                            assert!(false);
                        }
                    }
                } else {
                    assert!(false);
                }
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
    // TBD
}
