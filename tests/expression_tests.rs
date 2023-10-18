use std::{io::BufReader, fs::File};

use geoconverter::{
    parse,
    houdini_geo_schema_manipulator
};

#[test]
fn test_write_something() {
    let filepath = "tests/boxattr.bgeo";
    let mut f = File::open(filepath).expect("failed to open test file");
    let geo_data = parse(&mut BufReader::new(f));

    let mut manip = houdini_geo_schema_manipulator::HoudiniGeoSchemaManipulator::new(&geo_data);

    manip.run_over_point_attributes("@foo+100.29", "foo");

    println!("{:?}", manip.into_result());
}