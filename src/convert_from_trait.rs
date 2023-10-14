pub trait ConvertFromAll<T> {
    fn convert_from(val: T) -> Self;
}

macro_rules! _convert_from_filler {
    ($T:ty, $K:ty) => {
        impl ConvertFromAll<$T> for $K {
            fn convert_from(val: $T) -> Self {
                val as $K
            }
        }
    };
}

_convert_from_filler!(i8, i64);
_convert_from_filler!(i16, i64);
_convert_from_filler!(i32, i64);
_convert_from_filler!(i64, i64);
_convert_from_filler!(u8, i64);
_convert_from_filler!(u16, i64);
_convert_from_filler!(f32, i64);
_convert_from_filler!(f64, i64);

_convert_from_filler!(i8, f64);
_convert_from_filler!(i16, f64);
_convert_from_filler!(i32, f64);
_convert_from_filler!(i64, f64);
_convert_from_filler!(u8, f64);
_convert_from_filler!(u16, f64);
_convert_from_filler!(f32, f64);
_convert_from_filler!(f64, f64);

_convert_from_filler!(i8, usize);
_convert_from_filler!(i16, usize);
_convert_from_filler!(i32, usize);
_convert_from_filler!(i64, usize);
_convert_from_filler!(u8, usize);
_convert_from_filler!(u16, usize);
_convert_from_filler!(f32, usize);
_convert_from_filler!(f64, usize);
