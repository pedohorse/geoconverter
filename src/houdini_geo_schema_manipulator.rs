use crate::expressions::{self, evaluate_expression_precompiled_with_bindings, BindingValue, PrecompiledCode};
use crate::geo_struct::ReaderElement;
use crate::houdini_geo_schema::{
    GeoAttribute, GeoAttributeKind, HoudiniGeoSchemaParser, TupleGeoAttribute, TupleGeoAttributeChunk,
};
use std::num::NonZeroUsize;
use std::thread;

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
        let precomp = expressions::precompile_expression(expression);
        self.run_over_point_attributes_precompiled(&precomp, target_attribute_name);
    }

    pub fn run_over_point_attributes_precompiled(&mut self, precomp: &PrecompiledCode, target_attribute_name: &str) {
        self.schema_parser.parse_point_attributes();

        let target_attribute_kind = if let Some(x) = self.schema_parser.point_attribute(target_attribute_name) {
            x
        } else {
            panic!("no target point attribute '{}' found", target_attribute_name);
        };

        // TODO: this is all a prototype placeholder for now
        let mut bind_attrs = Vec::new();
        for attr_name in precomp.binding_names() {
            if let Some(attr) = self.schema_parser.point_attribute(attr_name) {
                bind_attrs.push(attr);
            } else {
                // panic for now, maybe TODO some defaults later
                panic!("requested attribute {} not found", attr_name);
            }
        }

        match target_attribute_kind {
            GeoAttributeKind::Float64(target_attr_source) => {
                let mut target_attr = target_attr_source.clone();
                
                let min_thread_chunk = 1024_usize;  // TODO: make a parameter !

                let max_threads: usize = std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN).into();

                // check if can and should multithread
                if max_threads > 1 && target_attr.len() > min_thread_chunk {
                    let thread_count = max_threads.min(target_attr.len() / min_thread_chunk);
                    let chunk_size = (target_attr.len() / thread_count).max(16);  // just sane min

                    let mut chunks = target_attr.chunks_mut_scoped(chunk_size);

                    thread::scope(|scope| {
                        let mut handles = Vec::new(); // TODO: with capacity

                        for chunk in chunks.iter_mut() {
                            let thread_handle = scope.spawn(|| {
                                TupleGeoAttributeChunk::run_over_f64(&precomp, chunk, &bind_attrs);
                            });
                            handles.push(thread_handle);
                        }
                        for handle in handles {
                            handle.join().expect("thread joinint failure!");
                        }
                    });
                } else {
                    // single thread
                    TupleGeoAttribute::run_over_f64(&precomp, &mut target_attr, &bind_attrs);
                }

                HoudiniGeoSchemaParser::write_to_strucutre(GeoAttributeKind::Float64(target_attr), &mut self.result_geo_data);
            }
            _ => {
                panic!("not yet implemented!");
            }
        }

        // HoudiniGeoSchemaParser::get_point_attrib_element_mut(&mut self.result_geo_data)
    }
}

///
/// trait to help implement same function for several similar types
///
trait RunOverF64Attribute<'b> {
    fn run_over_f64(
        precomp: &PrecompiledCode,
        target_attr: &mut Self,
        bind_attrs: &Vec<&GeoAttributeKind>,
    );
}

macro_rules! _helper_run_over_f64 {
    ($ftype:ty) => {
        impl<'b> RunOverF64Attribute<'b> for $ftype {
            fn run_over_f64(
                precomp: &PrecompiledCode,
                target_attr: &mut $ftype,
                bind_attrs: &Vec<&GeoAttributeKind>,
            ) {
                let mut values = precomp.clone_binding_values();

                for elem in target_attr.get_element_numbers_range() {
                    for (bvalue, attr_kind) in values.iter_mut().zip(bind_attrs.iter()) {
                        // TODO: this matching inside biiig loop is highly ineffective - surely we can match everything beforehand?
                        match attr_kind {
                            GeoAttributeKind::Float64(attr) => match attr.tuple_size() {
                                1 => {
                                    *bvalue = BindingValue::Float(attr.value(elem)[0]);
                                }
                                3 => {
                                    *bvalue = BindingValue::Vector3(attr.value(elem).try_into().expect("impossibru"));
                                }
                                i => {
                                    panic!("attrib tuples of size {} are not yet supported in expressions", i);
                                }
                            },
                            _ => {
                                panic!("not yet unplemented!");
                            }
                        }
                    }
                    let val =
                        evaluate_expression_precompiled_with_bindings(precomp, &values).expect("failed to evaluate expression");
                    match val {
                        BindingValue::Float(f) => target_attr.set_value(elem, &[f]),
                        BindingValue::Vector3(v) => target_attr.set_value(elem, &v.as_slice()[..target_attr.tuple_size()]),
                        _ => {
                            panic!("only f64 attribs are supported in this prototype");
                        }
                    };
                }
            }
        }
    };
}

_helper_run_over_f64!(TupleGeoAttributeChunk<'b, f64>);
_helper_run_over_f64!(TupleGeoAttribute<f64>);
