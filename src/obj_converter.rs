use crate::{
    houdini_geo_schema::{GeoAttribute, GeoAttributeKind, HoudiniGeoSchemaParser},
    ReaderElement,
};
use std::io::Write;

pub fn serialize_obj<F: ?Sized>(geo_schema: &mut HoudiniGeoSchemaParser, file: &mut F)
where
    F: Write,
{
    geo_schema.parse_primitives();
    geo_schema.parse_point_attributes();
    geo_schema.parse_vertex_attributes();

    let p_attr = if let Some(GeoAttributeKind::Float64(x)) = geo_schema.point_attribute("P") {
        x
    } else {
        panic!("no p float3 attr");
    };

    //writing P (Cd)
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

    // writing uv
    let have_uvs = if let Some(GeoAttributeKind::Float64(uv_attr)) = geo_schema.vertex_attribute("uv") {
        match uv_attr.tuple_size(){
            3 => {
                for vtxnum in 0..geo_schema.vertex_count() {
                    let uv = uv_attr.value(vtxnum);
                    file.write(format!("vt {} {} {}\n", uv[0], uv[1], uv[2]).as_bytes())
                        .expect("io error");
                }
                true
            }
            2 => {
                for vtxnum in 0..geo_schema.vertex_count() {
                    let uv = uv_attr.value(vtxnum);
                    file.write(format!("vt {} {} 0\n", uv[0], uv[1]).as_bytes())
                        .expect("io error");
                }
                true
            }
            x => {
                println!("float uv attribute found, but it's of unexpected size {}, skipping", x);
                false
            }
        }
    } else { false };

    // writing faces
    if have_uvs {
        for prim in geo_schema.polygons() {
            file.write(b"f").expect("io error");

            // obj expects opposite winding order starting at same vertex, and vertex indices start at 1, no 0
            let vtxcount = prim.vertices.len();
            for i in 0..vtxcount {
                let vtx = &prim.vertices[(vtxcount-i)%vtxcount];
                file.write(format!(" {}/{}", vtx.ptnum + 1, vtx.vtxnum + 1).as_bytes())
                    .expect("io error");
            }
            file.write(b"\n").expect("io error");
        }
    } else {
        for prim in geo_schema.polygons() {
            file.write(b"f").expect("io error");

            // obj expects opposite winding order starting at same vertex, and vertex indices start at 1, no 0
            let vtxcount = prim.vertices.len();
            for i in 0..vtxcount {
                let vtx = &prim.vertices[(vtxcount-i)%vtxcount];
                file.write(format!(" {}", vtx.ptnum + 1).as_bytes())
                    .expect("io error");
            }
            file.write(b"\n").expect("io error");
        }
    }
}
