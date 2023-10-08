use std::io::prelude::*;
use crate::houdini_geo_schema::{GeoAttribute, GeoAttributeKind, HoudiniGeoSchemaParser};

#[derive(Debug)]
pub struct StlSolid<T> {
    faces: Vec<StlFacet<T>>,
}

#[derive(Debug)]
pub struct StlFacet<T> {
    normal: [T; 3],
    vertices: [[T; 3]; 3],
}

pub fn create_stl_solid(geo_schema: &mut HoudiniGeoSchemaParser) -> StlSolid<f64> {
    let mut stl_faces = Vec::with_capacity(geo_schema.primitive_count()*2);  // this is ROUGH estimation (assume all prims are polys, all 4-gons)

    geo_schema.parse_point_attributes();
    geo_schema.parse_primitive_attributes();
    geo_schema.parse_primitives();

    let p_attr = if let Some(GeoAttributeKind::Float64(x)) = geo_schema.point_attribute("P") {
        x
    } else {
        panic!("unexpected P attrib type");
    };
    let n_attr = if let Some(GeoAttributeKind::Float64(x)) = geo_schema.primitive_attribute("N") {
        Some(x)
    } else {
        println!("no N attrib that is float3");
        None
    };

    for poly in geo_schema.polygons() {
        //println!("poly {:?}", poly);
        let mut vertex_iter = poly.vertices.iter();

        let first_vtx = vertex_iter.next().expect("no vertices?? bad polygon, BAD!");
        let second_vtx = vertex_iter
            .next()
            .expect("single vertex?? bad polygon, BAD!");
        let first_p: [f64; 3] = p_attr
            .value(first_vtx.ptnum)
            .try_into()
            .expect("bad P, not a float3");
        let mut prev_p = p_attr
            .value(second_vtx.ptnum)
            .try_into()
            .expect("bad P, not a float3");

        let n: [f64; 3] = if let Some(n_attr) = n_attr {
            n_attr
                .value(poly.number)
                .try_into()
                .expect("bad N, not a float3")
        } else {
            [0.0, 0.0, 0.0]
        };

        for vtx in vertex_iter {
            //println!("{:?}", p_attr.value(vtx.ptnum));

            let p = p_attr
                .value(vtx.ptnum)
                .try_into()
                .expect("bad P, not a float3");
            stl_faces.push(StlFacet {
                normal: [n[0], n[1], n[2]],
                vertices: [first_p, prev_p, p],
            });
            prev_p = p;
        }
    }

    StlSolid { faces: stl_faces }
}

pub fn serialize_stl<T, F>(stl_solid: &StlSolid<T>, file: &mut F)
where
    T: std::fmt::Display,
    F: Write,
{
    file.write(b"solid\n").expect("io error?");
    for facet in stl_solid.faces.iter() {
        file.write(
            format!(
                "facet normal {} {} {}\n",
                facet.normal[0], facet.normal[1], facet.normal[2]
            )
            .as_bytes(),
        )
        .expect("io error?");

        file.write(b"outer loop\n").expect("io error?");
        
        file.write(
            format!(
                "vertex {} {} {}\n\
                 vertex {} {} {}\n\
                 vertex {} {} {}\n",
                facet.vertices[0][0], facet.vertices[0][1], facet.vertices[0][2],
                facet.vertices[1][0], facet.vertices[1][1], facet.vertices[1][2],
                facet.vertices[2][0], facet.vertices[2][1], facet.vertices[2][2]
            )
            .as_bytes(),
        )
        .expect("io error?");
        
        file.write(b"endloop\nendfacet\n").expect("io error?");
    }
    file.write(b"endsolid\n").expect("io error?");
}
