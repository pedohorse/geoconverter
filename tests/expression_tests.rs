use std::{io::BufReader, fs::File};

use geoconverter::{
    parse,
    houdini_geo_schema_manipulator,
    HoudiniGeoSchemaParser,
    GeoAttributeKind,
    GeoAttribute
};

#[test]
fn test_write_something() {
    let filepath = "tests/boxattr.bgeo";
    let mut f = File::open(filepath).expect("failed to open test file");
    let geo_data = parse(&mut BufReader::new(f));

    let mut manip = houdini_geo_schema_manipulator::HoudiniGeoSchemaManipulator::new(&geo_data);

    manip.run_over_point_attributes("@foo+100.29", "foo");

    let result_elem = manip.into_result();

    // now test
    let mut result_parser = HoudiniGeoSchemaParser::new(&result_elem);

    result_parser.parse_point_attributes();

    let attr = if let Some(GeoAttributeKind::Float64(attr)) = result_parser.point_attribute("foo") {
        attr
    } else {
        panic!("no foo? wtf?")
    };

    // orig value in test file is stored as f32, then converted to f64, 
    // so we have to test the same way
    assert_eq!(100.29, attr.value(0)[0]);
    assert_eq!(100.29 + (1.23*1.0_f32) as f64, attr.value(1)[0]);
    assert_eq!(100.29 + (1.23*2.0_f32) as f64, attr.value(2)[0]);
    assert_eq!(100.29 + (1.23*4.0_f32) as f64, attr.value(4)[0]);    
}
