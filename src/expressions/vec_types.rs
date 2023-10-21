use std::{ops::{Add, Sub, Div, Mul, Index}, array::TryFromSliceError};



#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector<T, const N: usize> {
    data: [T; N]
}

macro_rules! implSelf {
    ($ftype:ty, $size:literal, $($x:ident),*) => {
        impl Vector<$ftype, $size> {
            pub fn new($($x: $ftype),*) -> Self {
                Vector {
                    data: [$($x),*]
                }
            }

            pub fn as_slice(&self) -> &[$ftype] {
                &self.data
            }
        }
    };
}

// 3f64
implSelf!(f64, 3, x, y, z);

// iter trait

macro_rules! implIndex {
    ($ftype:ty, $size:literal) => {
        impl Index<usize> for Vector<$ftype, $size> {
            type Output = $ftype;
            fn index(&self, index: usize) -> &Self::Output {
                &self.data[index]
            }
        }
    };
}

// 3f64
implIndex!(f64, 3);

// operation traits

macro_rules! implOp {
    ($opname:tt, $fname:tt, $ftype:ty, $size:literal, $op:tt) => {
        impl $opname<Self> for Vector<$ftype, $size> {
            type Output = Self;
        
            fn $fname(self, rhs: Self) -> Self::Output {
                let mut res: [$ftype; $size] = [<$ftype>::default(); $size];
                res.iter_mut().zip(self.data).zip(rhs.data).for_each(|((r, a), b)| {*r = a $op b});
                Self {
                    data: res
                }
            }
        }
    };
}

// 3f64
implOp!(Add, add, f64, 3, +);
implOp!(Sub, sub, f64, 3, -);
implOp!(Mul, mul, f64, 3, /);
implOp!(Div, div, f64, 3, *);


macro_rules! implOpScalar {
    ($opname:tt, $fname:tt, $ftype:ty, $size:literal, $op:tt, $stype:ty) => {
        impl $opname<$stype> for Vector<$ftype, $size> {
            type Output = Self;
        
            fn $fname(self, rhs: $stype) -> Self::Output {
                let mut res: [$ftype; $size] = [<$ftype>::default(); $size];
                res.iter_mut().zip(self.data).for_each(|(r, a)| {*r = a $op (rhs as $ftype)});
                Self {
                    data: res
                }
            }
        }
    };
}

// 3f64
implOpScalar!(Add, add, f64, 3, +, f64);
implOpScalar!(Sub, sub, f64, 3, -, f64);
implOpScalar!(Mul, mul, f64, 3, *, f64);
implOpScalar!(Div, div, f64, 3, /, f64);

// convertion

// 3f64
impl TryFrom<&[f64]> for Vector<f64, 3> {
    type Error = TryFromSliceError;
    fn try_from(value: &[f64]) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok(data) => Ok(Vector { data: data }),
            Err(e) => Err(e)
        }
    }
}