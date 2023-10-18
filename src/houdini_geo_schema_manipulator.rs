use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::ops::Deref;

use crate::expressions::{self, evaluate_expression_precompiled, PrecompiledCode, BindingValue, evaluate_expression_precompiled_with_bindings};
use crate::geo_struct::ReaderElement;
use crate::houdini_geo_schema::{HoudiniGeoSchemaParser, GeoAttributeKind, GeoAttribute};

pub struct HoudiniGeoSchemaManipulator<'a> {
    result_geo_data: ReaderElement,
    schema_parser: HoudiniGeoSchemaParser<'a>,
}

impl<'a> HoudiniGeoSchemaManipulator<'a> {
    pub fn new(geo_data: &'a ReaderElement) -> HoudiniGeoSchemaManipulator<'a> {
        HoudiniGeoSchemaManipulator {
            result_geo_data: geo_data.clone(),
            schema_parser: HoudiniGeoSchemaParser::new(&geo_data),
        }
    }

    pub fn into_result(self) -> ReaderElement {
        self.result_geo_data
    }

    pub fn run_over_point_attributes(&mut self, expression: &str, target_attribute_name: &str) {
        self.schema_parser.parse_point_attributes();

        let target_attribute_kind = if let Some(x) = self.schema_parser.point_attribute(target_attribute_name) {
            x
        } else {
            panic!("no target point attribute '{}' found", target_attribute_name);
        };

        let precomp = expressions::precompile_expression(expression);

        let mut values = precomp.clone_binding_values();

        // TODO: this is all a prototype placeholder for now
        let mut bind_attrs = Vec::new();
        for (bind_val, attr_name) in values.iter_mut().zip(precomp.binding_names()) {
            if let Some(GeoAttributeKind::Float64(attr)) = self.schema_parser.point_attribute(attr_name) {
                bind_attrs.push(attr);
            } else {
                // panic for now, maybe TODO some defaults later
                panic!("requested attribute {} not found", attr_name);
            }
        }

        match target_attribute_kind {
            GeoAttributeKind::Float64(target_attr_source) => {
                let mut target_attr = target_attr_source.clone();

                for elem in 0..target_attr.len() {
                    for (bvalue, attr) in values.iter_mut().zip(bind_attrs.iter()) {
                        *bvalue = BindingValue::Float(attr.value(elem)[0]);
                    }
                    let val = evaluate_expression_precompiled_with_bindings(&precomp, &values).expect("failed to evaluate expression");
                    match val {
                        BindingValue::Float(f) => target_attr.set_value(elem, &[f]),
                        _ => { panic!("only f64 attribs are supported in this prototype"); }
                    }
                }

                HoudiniGeoSchemaParser::write_to_strucutre(GeoAttributeKind::Float64(target_attr), &mut self.result_geo_data);
            }
            _ => { panic!("not yet implemented!"); }
        }
        
        
        // HoudiniGeoSchemaParser::get_point_attrib_element_mut(&mut self.result_geo_data)
        
    }
}
