use crate::{
    houdini_geo_schema::{GeoAttribute, GeoAttributeKind, HoudiniGeoSchemaParser},
    ReaderElement,
};
use std::io::Write;

pub fn serialize_obj<F>(geo_schema: &mut HoudiniGeoSchemaParser, file: &mut F)
where
    F: Write,
{
    geo_schema.parse_primitives();
    geo_schema.parse_point_attributes();

    let p_attr = if let Some(GeoAttributeKind::Float64(x)) = geo_schema.point_attribute("P") {
        x
    } else {
        panic!("no p float3 attr");
    };

    match geo_schema.point_attribute("Cd") {
        Some(GeoAttributeKind::Float64(cd_attr)) => {
            for ptnum in 0..geo_schema.point_count() {
                let p = p_attr.value(ptnum);
                let cd: &[f64] = cd_attr.value(ptnum);
                file.write(
                    format!(
                        "v {} {} {} {} {} {}\n",
                        p[0], p[1], p[2], cd[0], cd[1], cd[2]
                    )
                    .as_bytes(),
                )
                .expect("io error");
            }
        }
        None => {
            for ptnum in 0..geo_schema.point_count() {
                let p = p_attr.value(ptnum);
                file.write(format!("v {} {} {}\n", p[0], p[1], p[2]).as_bytes())
                    .expect("io error");
            }
        }
        _ => panic!("Cd present, but not a float3 attr"),
    };

    for prim in geo_schema.polygons() {
        file.write(b"f").expect("io error");

        // obj expects opposite winding order, and vertex indices start at 1, no 0
        for vtx in prim.vertices.iter().rev() {
            file.write(format!(" {}", vtx.ptnum + 1).as_bytes())
                .expect("io error");
        }
        file.write(b"\n").expect("io error");
    }
}
