// So far it's a very limited parser, just enough for stl, but with slight thought of the future

use std::collections::HashMap;

use crate::geo_struct::ReaderElement;

pub struct HoudiniGeoSchemaParser<'a> {
    structure: &'a ReaderElement,
    _point_attributes: Option<HashMap<&'a str, GeoAttributeKind>>,
    _vertex_attributes: Option<HashMap<&'a str, GeoAttributeKind>>,
    _prim_attributes: Option<HashMap<&'a str, GeoAttributeKind>>,
    _point_attribute_names_cached: Option<Vec<&'a str>>,
    _vertex_nums_to_point_nums: Vec<usize>,
    _polygons: Option<Vec<GeoPolygon>>,
    _prim_count: usize,
    _point_count: usize,
    _vertex_count: usize
}

pub struct TupleGeoAttribute<T: Copy> {
    tuple_size: usize,
    data: Vec<T>,
}

pub struct TokenGeoAttribute {
    tokens: Vec<String>,
    data: Vec<usize>,
}

pub trait GeoAttribute<'a, T: ?Sized> {
    fn value(&'a self, number: usize) -> &'a T;
}

impl<'a, T: Copy> GeoAttribute<'a, [T]> for TupleGeoAttribute<T> {
    fn value(&'a self, number: usize) -> &'a [T] {
        &self.data[number * self.tuple_size..(number + 1) * self.tuple_size]
    }
}

impl<'a> GeoAttribute<'a, String> for TokenGeoAttribute {
    fn value(&'a self, number: usize) -> &'a String {
        let shit = self.data[number];
        &self.tokens[shit]
    }
}

pub enum GeoAttributeKind {
    Float64(TupleGeoAttribute<f64>),
    Int64(TupleGeoAttribute<i64>),
    String(TokenGeoAttribute),
}

#[derive(Debug)]
pub struct GeoVertex {
    pub ptnum: usize,
    pub vtxnum: usize,
}

#[derive(Debug)]
pub struct GeoPolygon {
    pub number: usize,
    pub vertices: Vec<GeoVertex>,
}

fn get_from_kv_array<'a>(arr_elem: &'a ReaderElement, key: &str) -> Option<&'a ReaderElement> {
    let mut is_key = false;
    let mut next_one_is_the_shit = false;
    let arr = if let ReaderElement::Array(x) = arr_elem {
        x
    } else {
        panic!("bad schema! expected kv array, but it's not!");
    };

    for elem in arr.iter() {
        is_key = !is_key;
        if next_one_is_the_shit {
            return Some(elem);
        }
        if !is_key {
            continue;
        };
        if let ReaderElement::Text(arr_key) = elem {
            if arr_key == key {
                next_one_is_the_shit = true;
            }
        } else {
            panic!("bad schema! root array key is not a string {:?}", arr_elem)
        }
    }
    return None;
}

impl<'a> HoudiniGeoSchemaParser<'a> {
    pub fn new(read_structure: &'a ReaderElement) -> HoudiniGeoSchemaParser<'a> {
        let indices = if let Some(topo) = get_from_kv_array(read_structure, "topology") {
            if let Some(pointref) = get_from_kv_array(topo, "pointref") {
                if let Some(ReaderElement::Array(idxs)) = get_from_kv_array(pointref, "indices") {
                    Vec::from_iter(idxs.iter().map(|x| -> usize {
                        if let ReaderElement::Int(u) = x {
                            *u as usize
                        } else {
                            panic!("bad schema! index no int!");
                        }
                    }))
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let prim_count: usize = if let Some(ReaderElement::Int(x)) = get_from_kv_array(read_structure, "primitivecount") {
            (*x).try_into().expect("bad primitivecount")
        } else {
            panic!("bad schema! primitivecount not found");
        };
        let point_count: usize = if let Some(ReaderElement::Int(x)) = get_from_kv_array(read_structure, "pointcount") {
            (*x).try_into().expect("bad pointcount")
        } else {
            panic!("bad schema! pointcount not found");
        };
        let vertex_count: usize = if let Some(ReaderElement::Int(x)) = get_from_kv_array(read_structure, "vertexcount") {
            (*x).try_into().expect("bad vertexcount")
        } else {
            panic!("bad schema! vertexcount not found");
        };

        HoudiniGeoSchemaParser {
            structure: read_structure,
            _point_attributes: None,
            _vertex_attributes: None,
            _prim_attributes: None,
            _point_attribute_names_cached: None,
            _vertex_nums_to_point_nums: indices,
            _polygons: None,
            _prim_count: prim_count,
            _point_count: point_count,
            _vertex_count: vertex_count,
        }
    }

    fn parse_values<T>(
        values: &ReaderElement,
        tuple_size: usize,
        reader_element_mapper: &dyn Fn(&ReaderElement) -> T,
        starting_capacity: usize,
    ) -> Vec<T> {
        let mut attrib_values: Vec<T> = if starting_capacity == 0 {
            Vec::new()
        } else {
            Vec::with_capacity(
                starting_capacity
                    * if let Some(ReaderElement::Int(x)) = get_from_kv_array(values, "size") {
                        *x as usize
                    } else {
                        1
                    },
            )
        };

        if let Some(ReaderElement::Array(tuples)) = get_from_kv_array(values, "tuples") {
            // so it's key tuples
            for tuple in tuples {
                let tuple = if let ReaderElement::Array(x) = tuple {
                    x
                } else {
                    panic!("bad schema! value tuple is no tuple")
                };
                if tuple.len() != tuple_size {
                    panic!("bad schema! value tuple is not of declared size")
                }

                // TODO: wtf is this mess below? is it really the rust way to go?
                attrib_values.extend(tuple.iter().map(reader_element_mapper));
            }
        } else if let Some(ReaderElement::Array(arrays)) = get_from_kv_array(values, "arrays") {
            // so it's key arrays
            // for now i know only of case of size=1, no idea when it can be not 1 and what would that mean
            let values_size = if let Some(ReaderElement::Int(x)) = get_from_kv_array(values, "size")
            {
                *x
            } else {
                panic!("bad schema! no size in values");
            };
            // can values_size differ from tuple_size? haven't seen such cases
            if values_size != 1 {
                panic!("no idea how to parse values with arrays and size!=1");
            }
            let indices = if let ReaderElement::Array(x) = &arrays[0] {
                x
            } else {
                panic!("bad schema! arrays had no arrays!")
            };
            attrib_values.extend(indices.iter().map(reader_element_mapper));
        } else {
            panic!("rawpagedata support not implemented yet");
        }

        attrib_values
    }

    pub fn parse_point_attributes(&mut self) {
        self._point_attributes =
            Self::parse_attributes(self.structure, "pointattributes", "pointcount");
    }

    pub fn parse_vertex_attributes(&mut self) {
        self._vertex_attributes =
            Self::parse_attributes(self.structure, "vertexattributes", "vertexcount");
    }

    pub fn parse_primitive_attributes(&mut self) {
        self._prim_attributes =
            Self::parse_attributes(self.structure, "primitiveattributes", "primitivecount");
    }

    fn parse_attributes(
        structure: &'a ReaderElement,
        attrib_key: &str,
        elem_count_key: &str,
    ) -> Option<HashMap<&'a str, GeoAttributeKind>> {
        let mut attribute_map = HashMap::new();

        let elem_count: usize =
            if let Some(ReaderElement::Int(x)) = get_from_kv_array(structure, elem_count_key) {
                (*x).try_into().expect("incorrect element count")
            } else {
                panic!("could not find key {}", elem_count_key);
            };

        let attributes = if let Some(x) = get_from_kv_array(structure, "attributes") {
            x
        } else {
            panic!("bad schema! attributes must be an array");
        };
        let elem_attributes =
            if let Some(ReaderElement::Array(x)) = get_from_kv_array(attributes, attrib_key) {
                x
            } else {
                println!("no {} attributes!", attrib_key);
                return Some(HashMap::new());
            };

        for elem_attribute_block_el in elem_attributes {
            let elem_attribute_block = if let ReaderElement::Array(x) = elem_attribute_block_el {
                x
            } else {
                println!("bad schema! unrecognized point attribute block type, skipping");
                continue;
            };
            if elem_attribute_block.len() != 2 {
                println!("bad schema! unrecognized point attribute block type, skipping");
                continue;
            }
            let attrib_name = if let Some(ReaderElement::Text(x)) =
                get_from_kv_array(&elem_attribute_block[0], "name")
            {
                x
            } else {
                panic!("bad schema! no attrib name");
            };
            let attrib_type = if let Some(ReaderElement::Text(x)) =
                get_from_kv_array(&elem_attribute_block[0], "type")
            {
                x
            } else {
                panic!("bad schema! no attrib type");
            };

            let values = if let Some(x) = get_from_kv_array(&elem_attribute_block[1], "values") {
                x
            } else if let Some(x) = get_from_kv_array(&elem_attribute_block[1], "indices") {
                // for now treat indices same as values
                x
            } else {
                panic!("bad schema! attrib values bad");
            };

            // either it has tuples, or rawpagedata, or arrays

            attribute_map.insert(
                attrib_name.as_str(),
                match attrib_type.as_str() {
                    "numeric" => {
                        let size: usize = if let Some(ReaderElement::Int(x)) =
                            get_from_kv_array(values, "size")
                        {
                            *x as usize
                        } else {
                            panic!("bad schema! no size for values!");
                        };
                        let storage = if let Some(ReaderElement::Text(x)) =
                            get_from_kv_array(values, "storage")
                        {
                            x
                        } else {
                            panic!("bad schema! no storage for values!");
                        };
                        if storage.starts_with("fpreal") {
                            GeoAttributeKind::Float64(TupleGeoAttribute {
                                tuple_size: size,
                                data: Self::parse_values(
                                    values,
                                    size,
                                    &|x| -> f64 {
                                        if let ReaderElement::Float(f) = x {
                                            *f
                                        } else if let ReaderElement::Int(f) = x {
                                            *f as f64
                                        } else {
                                            panic!("bad schema! {:?}", x);
                                        }
                                    },
                                    elem_count,
                                ),
                            })
                        } else if storage.starts_with("int") {
                            GeoAttributeKind::Int64(TupleGeoAttribute {
                                tuple_size: size,
                                data: Self::parse_values(
                                    values,
                                    size,
                                    &|x| -> i64 {
                                        if let ReaderElement::Int(f) = x {
                                            *f
                                        } else {
                                            panic!("bad schema! {:?}", x);
                                        }
                                    },
                                    elem_count,
                                ),
                            })
                        } else {
                            println!("not implemented parsing attrib storage {}", storage);
                            continue;
                        }
                    }
                    "string" => {
                        let strings = if let Some(ReaderElement::Array(x)) =
                            get_from_kv_array(&elem_attribute_block[1], "strings")
                        {
                            x
                        } else {
                            panic!("bad schema! no stirngs for string attrib!")
                        };
                        GeoAttributeKind::String(TokenGeoAttribute {
                            tokens: Vec::from_iter(strings.iter().map(|x| -> String {
                                if let ReaderElement::Text(s) = x {
                                    s.to_owned()
                                } else {
                                    panic!("bad schema! strings contain not a string")
                                }
                            })),
                            data: Self::parse_values(
                                values,
                                1,
                                &|x| -> usize {
                                    if let ReaderElement::Int(f) = x {
                                        (*f).try_into().expect(&format!(
                                            "failed to convert int to usize {}",
                                            *f
                                        ))
                                    } else {
                                        panic!("bad schema! {:?}", x);
                                    }
                                },
                                elem_count,
                            ),
                        })
                    }
                    _ => {
                        println!("not implemented parsing attrib type {}", attrib_type);
                        continue;
                    }
                },
            );
        }

        Some(attribute_map)
    }

    pub fn parse_primitives(&mut self) {

        let mut polygons = Vec::with_capacity(self._prim_count);
        let mut cur_prim_num: usize = 0;
        let prim_blocks = if let Some(ReaderElement::Array(x)) =
            get_from_kv_array(self.structure, "primitives")
        {
            x
        } else {
            panic!("bad schema! no primitives entry")
        };

        for prim_block in prim_blocks {
            let prim_block_arr = if let ReaderElement::Array(x) = prim_block {
                x
            } else {
                panic!("bad schema! prim block is no array");
            };
            if prim_block_arr.len() != 2 {
                println!("skipping unexpected prim block scheme");
                continue;
            }

            let block_type = get_from_kv_array(&prim_block_arr[0], "type")
                .expect("bad schema! no prim block type!");
            if let ReaderElement::Text(type_text) = block_type {
                if type_text != "Polygon_run" {
                    println!("skipping block {}", type_text);
                    if type_text.ends_with("_run") {
                        let nprims_in_block = if let Some(ReaderElement::Int(x)) =
                            get_from_kv_array(&prim_block_arr[1], "nprimitives")
                        {
                            *x
                        } else {
                            panic!(
                                "bad schema! {} block is expected to have nprimitives key",
                                type_text
                            );
                        };
                        cur_prim_num += <i64 as TryInto<usize>>::try_into(nprims_in_block).expect(
                            &format!("bad data. nprimitives negative?? {}", nprims_in_block),
                        );
                    } else {
                        cur_prim_num += 1;
                    }
                    continue;
                }
            } else {
                panic!("bad schemd! primitive type is not a string");
            }

            let start_vertex = if let Some(ReaderElement::Int(x)) =
                get_from_kv_array(&prim_block_arr[1], "startvertex")
            {
                *x
            } else {
                panic!("unexpected type!")
            } as usize;
            // let num_prims = if let Some(ReaderElement::Int(x)) =
            //     get_from_kv_array(&prim_block_arr[1], "nprimitives")
            // {
            //     *x as usize
            // } else {
            //     panic!("unexpected type!")
            // };
            // it's either nvertices_rle or nvertices
            // nvertices_rle case
            if let Some(ReaderElement::Array(vtx_cnt_pairs)) =
                get_from_kv_array(&prim_block_arr[1], "nvertices_rle")
            {
                // TODO: check len is even
                let mut vtx_cnts = vtx_cnt_pairs.iter();
                let mut cur_vtx = start_vertex;
                loop {
                    let vtx_cnt = if let Some(x) = vtx_cnts.next() {
                        if let ReaderElement::Int(u) = x {
                            *u as usize
                        } else {
                            panic!("bad schema! rel no int");
                        }
                    } else {
                        break;
                    };
                    let count = if let Some(ReaderElement::Int(u)) = vtx_cnts.next() {
                        *u as usize
                    } else {
                        panic!("bad schema! rle no even")
                    };

                    for _ in 0..count {
                        let mut prim_vtxs = Vec::with_capacity(vtx_cnt);
                        for vtx_shift in 0..vtx_cnt {
                            let vtx_num = cur_vtx + vtx_shift;
                            prim_vtxs.push(GeoVertex {
                                ptnum: self.vtx_to_ptnum(vtx_num),
                                vtxnum: vtx_num,
                            });
                        }
                        cur_vtx += vtx_cnt;
                        polygons.push(GeoPolygon {
                            number: cur_prim_num,
                            vertices: prim_vtxs,
                        });
                        cur_prim_num += 1;
                    }
                }
            } else
            // nvertices case
            if let Some(ReaderElement::Array(vtx_cnts)) =
                get_from_kv_array(&prim_block_arr[1], "nvertices")
            {
                let mut cur_vtx = start_vertex;
                for vtx_cnt in vtx_cnts {
                    let vtx_cnt = if let ReaderElement::Int(u) = vtx_cnt {
                        *u as usize
                    } else {
                        panic!("schema error! vtx cnt no int");
                    };

                    let mut prim_vtxs = Vec::with_capacity(vtx_cnt);
                    for vtx_shift in 0..vtx_cnt {
                        let vtx_num = cur_vtx + vtx_shift;
                        prim_vtxs.push(GeoVertex {
                            ptnum: self.vtx_to_ptnum(vtx_num),
                            vtxnum: vtx_num,
                        });
                    }
                    cur_vtx += vtx_cnt;
                    polygons.push(GeoPolygon {
                        number: cur_prim_num,
                        vertices: prim_vtxs,
                    });
                    cur_prim_num += 1;
                }
            } else {
                panic!("unexpected type!")
            };
        }

        self._polygons = Some(polygons);
    }

    pub fn point_attribute_names(&'a self) -> impl Iterator + 'a {
        let attrib_map = if let Some(x) = &self._point_attributes {
            x
        } else {
            panic!("point attributes were not parsed!");
        };

        attrib_map.keys()
    }

    pub fn point_attribute(&self, name: &str) -> Option<&GeoAttributeKind> {
        let attrib_map = if let Some(x) = &self._point_attributes {
            x
        } else {
            panic!("point attributes were not parsed!");
        };

        attrib_map.get(name)
    }

    pub fn primitive_attribute(&self, name: &str) -> Option<&GeoAttributeKind> {
        let attrib_map = if let Some(x) = &self._prim_attributes {
            x
        } else {
            panic!("prim attributes were not parsed!");
        };

        attrib_map.get(name)
    }

    pub fn vtx_to_ptnum(&self, vtx_num: usize) -> usize {
        self._vertex_nums_to_point_nums[vtx_num]
    }

    pub fn polygons(&self) -> &[GeoPolygon] {
        if let Some(p) = &self._polygons {
            return p;
        } else {
            panic!("primitives were not parsed!")
        }
    }

    pub fn primitive_count(&self) -> usize {
        self._prim_count
    }

    pub fn point_count(&self) -> usize {
        self._point_count
    }

    pub fn vertex_count(&self) -> usize {
        self._vertex_count
    }
}
