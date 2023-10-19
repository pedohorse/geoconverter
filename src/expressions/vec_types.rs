use std::ops::{Add, Sub, Div, Mul, Index};



#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector<T, const N: usize> {
    data: [T; N]
}

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

implIndex!(f64, 3);


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

implOpScalar!(Add, add, f64, 3, +, f64);
implOpScalar!(Sub, sub, f64, 3, -, f64);
implOpScalar!(Mul, mul, f64, 3, *, f64);
implOpScalar!(Div, div, f64, 3, /, f64);