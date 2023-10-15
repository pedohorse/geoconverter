// So far it's a very limited parser, just enough for stl, but with slight thought of the future

use std::collections::HashMap;

use crate::convert_from_trait::ConvertFromAll;
use crate::geo_struct::{ReaderElement, UniformArrayType};

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
    _vertex_count: usize,
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
    fn tuple_size(&'a self) -> usize;
    fn value(&'a self, number: usize) -> &'a T;
}

impl<'a, T: Copy> GeoAttribute<'a, [T]> for TupleGeoAttribute<T> {
    fn value(&'a self, number: usize) -> &'a [T] {
        &self.data[number * self.tuple_size..(number + 1) * self.tuple_size]
    }

    fn tuple_size(&'a self) -> usize{
        self.tuple_size
    }
}

impl<'a> GeoAttribute<'a, String> for TokenGeoAttribute {
    fn value(&'a self, number: usize) -> &'a String {
        let shit = self.data[number];
        &self.tokens[shit]
    }

    fn tuple_size(&'a self) -> usize{
        1
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
    // TODO: not optimal, better refactor with macros
    get_from_any_kv_array(arr_elem, &[key])
}

fn get_from_any_kv_array<'a>(arr_elem: &'a ReaderElement, keys: &[&str]) -> Option<&'a ReaderElement> {
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
            if let Some(_) = keys.iter().find(|&&x| x == arr_key) {
                // arr_key == key
                next_one_is_the_shit = true;
            }
        } else {
            panic!("bad schema! root array key is not a string {:?}", arr_elem)
        }
    }
    return None;
}

impl<'a> HoudiniGeoSchemaParser<'a> {
    
    /// construct new instance of HoudiniGeoSchemaParser
    /// 
    /// note that to access element attributes and primitives 
    /// you have to parse them beforehand explicitly
    /// 
    /// you should only parse what you need for particular geo conversion
    pub fn new(read_structure: &'a ReaderElement) -> HoudiniGeoSchemaParser<'a> {
        let indices = if let Some(topo) = get_from_kv_array(read_structure, "topology") {
            if let Some(pointref) = get_from_kv_array(topo, "pointref") {
                match get_from_kv_array(pointref, "indices") {
                    Some(ReaderElement::Array(idxs)) => Vec::from_iter(idxs.iter().map(|x| -> usize {
                        if let ReaderElement::Int(u) = x {
                            *u as usize
                        } else {
                            panic!("bad schema! index no int!");
                        }
                    })),
                    Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi8(vec))) => {
                        vec.iter().map(|x| -> usize { *x as usize }).collect()
                    }
                    Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi16(vec))) => {
                        vec.iter().map(|x| -> usize { *x as usize }).collect()
                    }
                    Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi32(vec))) => {
                        vec.iter().map(|x| -> usize { *x as usize }).collect()
                    }
                    Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi64(vec))) => {
                        vec.iter().map(|x| -> usize { *x as usize }).collect()
                    }
                    Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTu8(vec))) => {
                        vec.iter().map(|x| -> usize { *x as usize }).collect()
                    }
                    Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTu16(vec))) => {
                        vec.iter().map(|x| -> usize { *x as usize }).collect()
                    }
                    Some(_) => panic!("bad schema! index no int!"), // TODO impl uniform arr
                    None => Vec::new(),
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

    /// parse values into a linear array of target type
    /// values in geo file may look somewhat like this:
    /// "values",[
    ///    "size",3,
    ///    "storage","fpreal32",
    ///    "tuples",[[0.5,-0.5,0.5],[-0.5,-0.5,0.5],[0.5,0.5,0.5],[-0.5,0.5,0.5],[-0.5,
    ///  ]
    /// 
    /// or it can be called "incides", but be very similar in structure:
    /// "indices",[
    ///    "size",1,
    ///    "storage","int32",
    ///    "arrays",[[0,0,0,0,0,0,0,0]]
    /// ]
    /// 
    /// or, mostly in bgeo case, it will have rawpagedata, see function below that deals with that
    /// 
    /// even if element is tuple, we still produce liear array here, an later will use tuple_size for proper indexing
    fn parse_values<T>(
        values: &ReaderElement,
        tuple_size: usize,
        reader_element_mapper: &dyn Fn(&ReaderElement) -> T,
        number_of_elements: usize,
    ) -> Vec<T>
    where
        T: Copy
            + Default
            + ConvertFromAll<f32>
            + ConvertFromAll<f64>
            + ConvertFromAll<u8>
            + ConvertFromAll<u16>
            + ConvertFromAll<i8>
            + ConvertFromAll<i16>
            + ConvertFromAll<i32>
            + ConvertFromAll<i64>,
    {
        let mut attrib_values: Vec<T> = if number_of_elements == 0 {
            Vec::new()
        } else {
            Vec::with_capacity(
                number_of_elements
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
            let values_size = if let Some(ReaderElement::Int(x)) = get_from_kv_array(values, "size") {
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
        } else if let Some(raw_page_data) = get_from_kv_array(values, "rawpagedata") {
            let segment = match raw_page_data {
                ReaderElement::UniformArray(UniformArrayType::UniformArrayTf16(vec)) => {
                    Self::parse_rawpagedata::<T, f32>(values, vec, number_of_elements, tuple_size)
                }
                ReaderElement::UniformArray(UniformArrayType::UniformArrayTf32(vec)) => {
                    Self::parse_rawpagedata::<T, f32>(values, vec, number_of_elements, tuple_size)
                }
                ReaderElement::UniformArray(UniformArrayType::UniformArrayTf64(vec)) => {
                    Self::parse_rawpagedata::<T, f64>(values, vec, number_of_elements, tuple_size)
                }
                ReaderElement::UniformArray(UniformArrayType::UniformArrayTi8(vec)) => {
                    Self::parse_rawpagedata::<T, i8>(values, vec, number_of_elements, tuple_size)
                }
                ReaderElement::UniformArray(UniformArrayType::UniformArrayTi16(vec)) => {
                    Self::parse_rawpagedata::<T, i16>(values, vec, number_of_elements, tuple_size)
                }
                ReaderElement::UniformArray(UniformArrayType::UniformArrayTi32(vec)) => {
                    Self::parse_rawpagedata::<T, i32>(values, vec, number_of_elements, tuple_size)
                }
                ReaderElement::UniformArray(UniformArrayType::UniformArrayTi64(vec)) => {
                    Self::parse_rawpagedata::<T, i64>(values, vec, number_of_elements, tuple_size)
                }
                ReaderElement::UniformArray(UniformArrayType::UniformArrayTu8(vec)) => {
                    Self::parse_rawpagedata::<T, u8>(values, vec, number_of_elements, tuple_size)
                }
                ReaderElement::UniformArray(UniformArrayType::UniformArrayTu16(vec)) => {
                    Self::parse_rawpagedata::<T, u16>(values, vec, number_of_elements, tuple_size)
                }
                _ => {
                    panic!("Unexpected array type");
                }
            };
            attrib_values.extend(segment);
        } else {
            panic!("rawpagedata support not implemented yet");
        }

        attrib_values
    }

    /// this function deals with rawpagedata
    /// 
    ///  typically it looks like this:
    /// "values", [
    ///    "size", 3,
    ///    "storage", "fpreal32",
    ///    "packing", [3],
    ///    "pagesize", 1024,
    ///    "constantpageflags", [[]],
    ///    "rawpagedata", [0.5, -0.5, 0.5, -0.]
    /// 
    /// see details read minimal explanation here: https://www.sidefx.com/docs/hdk/_h_d_k__g_a__using.html#HDK_GA_FileFormat
    /// 
    /// raw_page_data must be from values. it's in args just to not get it twice
    fn parse_rawpagedata<T: Copy + Default + ConvertFromAll<K>, K: Copy>(
        values: &ReaderElement,
        raw_page_array: &Vec<K>,
        number_of_elements: usize,
        tuple_size: usize,
    ) -> Vec<T> {
        // rawpackagedata case

        let size = if let Some(ReaderElement::Int(x)) = get_from_kv_array(values, "size") {
            *x as usize
        } else {
            panic!("bad schemd! vector size not found!");
        };
        let page_size = if let Some(ReaderElement::Int(x)) = get_from_kv_array(values, "pagesize") {
            *x as usize
        } else {
            panic!("bad schemd! vector size not found!");
        };
        let packing = match get_from_kv_array(values, "packing") {
            Some(ReaderElement::Array(packing_array)) => {
                packing_array
                    .iter()
                    .map(|x| {
                        match x {
                            ReaderElement::Int(v) => *v as usize,
                            ReaderElement::Bool(v) => *v as usize, // wtf is this case even possible?
                            _ => {
                                panic!("bad schema! unexpected data type in packing element array");
                            }
                        }
                    })
                    .collect()
            }
            Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTu8(v))) => {
                v.into_iter().map(|x| *x as usize).collect()
            }
            Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTu16(v))) => {
                v.into_iter().map(|x| *x as usize).collect()
            }
            Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi8(v))) => {
                v.into_iter().map(|x| *x as usize).collect()
            }
            Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi16(v))) => {
                v.into_iter().map(|x| *x as usize).collect()
            }
            Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi32(v))) => {
                v.into_iter().map(|x| *x as usize).collect()
            }
            Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi64(v))) => {
                v.into_iter().map(|x| *x as usize).collect()
            }
            None => vec![size],
            _ => {
                panic!("bad schema! unknown packing data representation");
            }
        };
        let constant_page_flags: Vec<Vec<bool>> =
            if let Some(ReaderElement::Array(flag_array)) = get_from_kv_array(values, "constantpageflags") {
                let mut page_flags = Vec::with_capacity(packing.len());

                for flag_subvector in flag_array {
                    page_flags.push(match flag_subvector {
                        ReaderElement::Array(vec) => vec
                            .iter()
                            .map(|x| -> bool {
                                match x {
                                    ReaderElement::Bool(v) => *v,
                                    ReaderElement::Int(v) => *v != 0,
                                    _ => {
                                        panic!("bad schema! packing subvector has unexpected element {:?}", x);
                                    }
                                }
                            })
                            .collect(),
                        ReaderElement::UniformArray(UniformArrayType::UniformArrayTbool(vec)) => {
                            vec.clone()  // TODO: this may be a reference instead, but then prev arm need to be rethought
                        },
                        _ => {
                            panic!("bad schema! element of constantpageflags is not an array")
                        }
                    });
                }

                page_flags
            } else {
                vec![vec![]]
            };

        let mut cur_page = 0;
        let mut elements_left = number_of_elements;
        let mut cur_array_i = 0;

        let mut result: Vec<T> = Vec::with_capacity(number_of_elements * tuple_size);
        result.resize(number_of_elements * tuple_size, T::default());

        while elements_left > 0 {
            let base_idx = page_size * cur_page * tuple_size;
            let mut base_subvec_i = 0;

            // for each subvector as defined by packing info
            for (cur_subvector, subvec_size) in packing.iter().enumerate() {
                if constant_page_flags[cur_subvector].len() == 0 || !constant_page_flags[cur_subvector][cur_page] {
                    // if page flags empty or flag shows non-constant page

                    // copy entire page data
                    for i in 0..page_size.min(elements_left) {
                        let base = base_idx + i * tuple_size + base_subvec_i;
                        for k in 0..*subvec_size {
                            result[base..base + subvec_size][k] =
                                T::convert_from(raw_page_array[cur_array_i..cur_array_i + subvec_size][k]);
                        }
                        cur_array_i += subvec_size;
                    }
                    base_subvec_i += subvec_size;
                } else {
                    // the page is constant for subvector, it contains a single value
                    let val: Vec<T> = raw_page_array[cur_array_i..cur_array_i + subvec_size]
                        .iter()
                        .map(|x| T::convert_from(*x))
                        .collect();
                    cur_array_i += subvec_size;
                    // fill entire page data with same value
                    for i in 0..page_size.min(elements_left) {
                        let base = base_idx + i * tuple_size + base_subvec_i;
                        result[base..base + subvec_size].copy_from_slice(&val);
                    }
                }
            }
            elements_left -= page_size.min(elements_left);
            cur_page += 1;
        }

        assert_eq!(cur_array_i, raw_page_array.len()); // sanity check

        result
    }

    /// parse point attributes
    /// 
    pub fn parse_point_attributes(&mut self) {
        self._point_attributes = Self::parse_attributes(self.structure, "pointattributes", "pointcount");
    }

    /// parse vertex attributes
    /// 
    pub fn parse_vertex_attributes(&mut self) {
        self._vertex_attributes = Self::parse_attributes(self.structure, "vertexattributes", "vertexcount");
    }

    /// parse primitive attributes
    /// 
    pub fn parse_primitive_attributes(&mut self) {
        self._prim_attributes = Self::parse_attributes(self.structure, "primitiveattributes", "primitivecount");
    }

    /// parse general attribute structure
    /// 
    /// * `structure` - overall schema
    /// * `attrib_key` - name of the key where to find attributes
    /// * `elem_count_key` - key name of where to find element count
    /// 
    fn parse_attributes(
        structure: &'a ReaderElement,
        attrib_key: &str,
        elem_count_key: &str,
    ) -> Option<HashMap<&'a str, GeoAttributeKind>> {
        let mut attribute_map = HashMap::new();

        let elem_count: usize = if let Some(ReaderElement::Int(x)) = get_from_kv_array(structure, elem_count_key) {
            (*x).try_into().expect("incorrect element count")
        } else {
            panic!("could not find key {}", elem_count_key);
        };

        let attributes = if let Some(x) = get_from_kv_array(structure, "attributes") {
            x
        } else {
            panic!("bad schema! attributes must be an array");
        };
        let elem_attributes = if let Some(ReaderElement::Array(x)) = get_from_kv_array(attributes, attrib_key) {
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
            let attrib_name = if let Some(ReaderElement::Text(x)) = get_from_kv_array(&elem_attribute_block[0], "name") {
                x
            } else {
                panic!("bad schema! no attrib name");
            };
            let attrib_type = if let Some(ReaderElement::Text(x)) = get_from_kv_array(&elem_attribute_block[0], "type") {
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
                        let size: usize = if let Some(ReaderElement::Int(x)) = get_from_kv_array(values, "size") {
                            *x as usize
                        } else {
                            panic!("bad schema! no size for values!");
                        };
                        let storage = if let Some(ReaderElement::Text(x)) = get_from_kv_array(values, "storage") {
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
                        let strings =
                            if let Some(ReaderElement::Array(x)) = get_from_kv_array(&elem_attribute_block[1], "strings") {
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
                                        (*f).try_into().expect(&format!("failed to convert int to usize {}", *f))
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

    /// parse primitives from geo structure
    /// 
    /// for now only polygons are supported
    /// 
    /// TODO: support other types of primitives
    pub fn parse_primitives(&mut self) {
        let mut polygons = Vec::with_capacity(self._prim_count);
        let mut cur_prim_num: usize = 0;
        let prim_blocks = if let Some(ReaderElement::Array(x)) = get_from_kv_array(self.structure, "primitives") {
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

            let block_type = get_from_kv_array(&prim_block_arr[0], "type").expect("bad schema! no prim block type!");
            if let ReaderElement::Text(type_text) = block_type {
                // skip unknown for now types of blocks
                if type_text != "Polygon_run" && type_text != "p_r" {
                    println!("skipping block {}", type_text);
                    if type_text.ends_with("_run") {
                        let nprims_in_block = if let Some(ReaderElement::Int(x)) =
                            get_from_any_kv_array(&prim_block_arr[1], &["nprimitives", "n_p"])
                        {
                            *x
                        } else {
                            panic!("bad schema! {} block is expected to have nprimitives/n_p key", type_text);
                        };
                        cur_prim_num += <i64 as TryInto<usize>>::try_into(nprims_in_block)
                            .expect(&format!("bad data. nprimitives negative?? {}", nprims_in_block));
                    } else {
                        cur_prim_num += 1;
                    }
                    continue;
                }
            } else {
                panic!("bad schemd! primitive type is not a string");
            }

            let start_vertex =
                if let Some(ReaderElement::Int(x)) = get_from_any_kv_array(&prim_block_arr[1], &["startvertex", "s_v"]) {
                    *x
                } else {
                    panic!("unexpected type!")
                } as usize;

            // it's either nvertices_rle or nvertices

            // nvertices_rle case
            macro_rules! _loop_iter_helper {
                ($vtx_cnt_pairs:ident, $vtx_elem_func:expr) => {
                    let mut vtx_cnts = $vtx_cnt_pairs.iter();
                    let mut cur_vtx = start_vertex;
                    loop {
                        let vtx_cnt = if let Some(x) = vtx_cnts.next() {
                            $vtx_elem_func(x)
                        } else {
                            break;
                        };
                        let count = if let Some(x) = vtx_cnts.next() {
                            $vtx_elem_func(x)
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
                };
            }

            match get_from_any_kv_array(&prim_block_arr[1], &["nvertices_rle", "r_v"]) {
                Some(ReaderElement::Array(vtx_cnt_pairs)) => {
                    // TODO: check len is even
                    _loop_iter_helper!(vtx_cnt_pairs, |x: &ReaderElement| {
                        if let ReaderElement::Int(u) = x {
                            *u as usize
                        } else {
                            panic!("bad schema! rel no int");
                        }
                    });
                }
                Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTu8(vtx_cnt_pairs))) => {
                    _loop_iter_helper!(vtx_cnt_pairs, |x: &u8| { *x as usize });
                }
                Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTu16(vtx_cnt_pairs))) => {
                    _loop_iter_helper!(vtx_cnt_pairs, |x: &u16| { *x as usize });
                }
                Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi8(vtx_cnt_pairs))) => {
                    _loop_iter_helper!(vtx_cnt_pairs, |x: &i8| { *x as usize });
                }
                Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi16(vtx_cnt_pairs))) => {
                    _loop_iter_helper!(vtx_cnt_pairs, |x: &i16| { *x as usize });
                }
                Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi32(vtx_cnt_pairs))) => {
                    _loop_iter_helper!(vtx_cnt_pairs, |x: &i32| { *x as usize });
                }
                Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi64(vtx_cnt_pairs))) => {
                    _loop_iter_helper!(vtx_cnt_pairs, |x: &i64| { *x as usize });
                }
                None => {
                    // nvertices case
                    macro_rules! _loop_iter_helper2 {
                        ($vtx_cnts:ident, $vtx_cnt_func:expr) => {
                            let mut cur_vtx = start_vertex;
                            for vtx_cnt in $vtx_cnts {
                                let vtx_cnt = $vtx_cnt_func(vtx_cnt);

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
                        };
                    }

                    match get_from_any_kv_array(&prim_block_arr[1], &["nvertices", "n_v"]) {
                        Some(ReaderElement::Array(vtx_cnts)) => {
                            _loop_iter_helper2!(vtx_cnts, |v: &ReaderElement| {
                                if let ReaderElement::Int(u) = v {
                                    *u as usize
                                } else {
                                    panic!("schema error! vtx cnt no int");
                                }
                            });
                        }
                        Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTu8(vtx_cnts))) => {
                            _loop_iter_helper2!(vtx_cnts, |v: &u8| { *v as usize });
                        }
                        Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTu16(vtx_cnts))) => {
                            _loop_iter_helper2!(vtx_cnts, |v: &u16| { *v as usize });
                        }
                        Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi8(vtx_cnts))) => {
                            _loop_iter_helper2!(vtx_cnts, |v: &i8| { *v as usize });
                        }
                        Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi16(vtx_cnts))) => {
                            _loop_iter_helper2!(vtx_cnts, |v: &i16| { *v as usize });
                        }
                        Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi32(vtx_cnts))) => {
                            _loop_iter_helper2!(vtx_cnts, |v: &i32| { *v as usize });
                        }
                        Some(ReaderElement::UniformArray(UniformArrayType::UniformArrayTi64(vtx_cnts))) => {
                            _loop_iter_helper2!(vtx_cnts, |v: &i64| { *v as usize });
                        }
                        _ => {
                            panic!("unexpected type of nvertices/n_v block!");
                        }
                    }
                }
                _ => {
                    panic!("unexpected type of nvertices_rle/r_v block!");
                }
            }
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

    /// get vertex attribute
    ///
    /// vertex attributes have to be parsed beforehand
    pub fn vertex_attribute(&self, name: &str) -> Option<&GeoAttributeKind> {
        let attrib_map = if let Some(x) = &self._vertex_attributes {
            x
        } else {
            panic!("vertex attributes were not parsed!");
        };

        attrib_map.get(name)
    }

    /// get point attribute
    ///
    /// point attributes have to be parsed beforehand
    pub fn point_attribute(&self, name: &str) -> Option<&GeoAttributeKind> {
        let attrib_map = if let Some(x) = &self._point_attributes {
            x
        } else {
            panic!("point attributes were not parsed!");
        };

        attrib_map.get(name)
    }

    /// get primitive attribute
    ///
    /// primitive attributes have to be parsed beforehand
    pub fn primitive_attribute(&self, name: &str) -> Option<&GeoAttributeKind> {
        let attrib_map = if let Some(x) = &self._prim_attributes {
            x
        } else {
            panic!("prim attributes were not parsed!");
        };

        attrib_map.get(name)
    }

    /// get point number of the point given vertex belongs to
    /// 
    pub fn vtx_to_ptnum(&self, vtx_num: usize) -> usize {
        self._vertex_nums_to_point_nums[vtx_num]
    }

    /// get polygons
    /// 
    /// primitives have to be parsed beforehand
    pub fn polygons(&self) -> &[GeoPolygon] {
        if let Some(p) = &self._polygons {
            return p;
        } else {
            panic!("primitives were not parsed!")
        }
    }

    /// get primitive count
    /// 
    pub fn primitive_count(&self) -> usize {
        self._prim_count
    }

    /// get point count
    /// 
    pub fn point_count(&self) -> usize {
        self._point_count
    }

    /// get vertex count
    /// 
    pub fn vertex_count(&self) -> usize {
        self._vertex_count
    }
}
