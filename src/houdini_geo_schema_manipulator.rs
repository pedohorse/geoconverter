use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::ops::Deref;

use crate::expressions::{self, evaluate_expression_precompiled, evaluate_postfix, BindingValue};
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

        let (postfix, mut bindings_map) = expressions::precompile_expression(expression);

        // TODO: this is all a prototype placeholder for now
        let mut bind_attr_pairs = Vec::new();
        let mut target_attr = match target_attribute_kind {
            GeoAttributeKind::Float64(target_attr) => {
                for (key, binding) in bindings_map.iter_mut() {
                    if let Some(GeoAttributeKind::Float64(attr)) = self.schema_parser.point_attribute(&binding.name) {
                        bind_attr_pairs.push((*key, attr));
                    } else {
                        // panic for now, maybe TODO some defaults later
                        panic!("requested attribute {} not found", &binding.name);
                    }
                };
                target_attr.clone()
            }
            _ => { panic!("only f64 attribs are supported in this prototype"); }
        };

        for elem in 0..target_attr.len() {
            for (bid, attr) in bind_attr_pairs.iter_mut() {
                bindings_map.get_mut(bid).expect("impossibry")
                            .value = BindingValue::Float(attr.value(elem)[0]);
            }
            let val = evaluate_postfix(&postfix, &bindings_map).expect("failed to evaluate expression");
            match val {
                BindingValue::Float(f) => target_attr.set_value(elem, &[f]),
                _ => { panic!("only f64 attribs are supported in this prototype"); }
            }
        }
        
        HoudiniGeoSchemaParser::write_to_strucutre(GeoAttributeKind::Float64(target_attr), &mut self.result_geo_data);
        // HoudiniGeoSchemaParser::get_point_attrib_element_mut(&mut self.result_geo_data)
        
    }
}
