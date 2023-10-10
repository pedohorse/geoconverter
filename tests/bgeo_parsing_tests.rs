use geoconverter::{self, ReaderElement};

use std::fs::File;
use std::io::BufReader;

#[test]
fn parse_bgeo_box() {
    let mut f = File::open("./tests/box.bgeo").expect("failed to open test file");
    let elem = geoconverter::parse_binary(&mut BufReader::new(&mut f));

    geoconverter::preview(&elem);

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
