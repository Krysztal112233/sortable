//! Column-level type machinery: kinds and their mapping from Rust types.

use std::cmp::Ordering;
use std::fmt;

/// Type tag of a column: which Rust type its cells hold. Derived from the
/// row type at compile time (see [`NamedRow::KINDS`](crate::NamedRow::KINDS)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum ColumnKind {
    F64,  F32,
    I128, I16, I32, I64, I8, ISize,
    U128, U16, U32, U64, U8, USize,
    String,
    Bool,
    /// A nullable column: the kind of the inner, non-`Option` type.
    Optional(&'static ColumnKind),
}

impl fmt::Display for ColumnKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// A Rust type mapping to exactly one [`ColumnKind`], used for typed
/// borrowed access to cell values and for deriving a table's column kinds.
/// All implementors are owned, `'static`, cloneable, totally ordered types.
pub trait ColumnType: Sized + 'static + Clone {
    const KIND: ColumnKind;

    /// Total order used for sorting: floats compare via `total_cmp` (NaN
    /// has a defined place); `None` sorts before `Some`.
    fn cmp(a: &Self, b: &Self) -> Ordering;
}

macro_rules! impl_column_type {
   ($($t:ty => $variant:ident via $cmp:path),* $(,)?) => {$(
       impl ColumnType for $t {
           const KIND: ColumnKind = ColumnKind::$variant;

           fn cmp(a: &Self, b: &Self) -> Ordering { $cmp(a, b) }
       }
   )*};
}

#[rustfmt::skip]
impl_column_type! {
    f64    =>    F64 via f64::total_cmp, f32 => F32 via f32::total_cmp,
    i128   =>   I128 via Ord::cmp,       i16 => I16 via Ord::cmp,       i32   =>   I32 via Ord::cmp,
    i64    =>    I64 via Ord::cmp,       i8  =>  I8 via Ord::cmp,       isize => ISize via Ord::cmp,
    u128   =>   U128 via Ord::cmp,       u16 => U16 via Ord::cmp,       u32   =>   U32 via Ord::cmp,
    u64    =>    U64 via Ord::cmp,       u8  =>  U8 via Ord::cmp,       usize => USize via Ord::cmp,
    bool   =>   Bool via Ord::cmp,
    String => String via Ord::cmp,
}

/// `Option<T>` cells are nullable; the kind wraps the inner kind.
impl<T: ColumnType> ColumnType for Option<T> {
    const KIND: ColumnKind = ColumnKind::Optional(&T::KIND);

    fn cmp(a: &Self, b: &Self) -> Ordering {
        match (a, b) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(x), Some(y)) => T::cmp(x, y),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_of_plain_and_option_types() {
        assert_eq!(<f32 as ColumnType>::KIND, ColumnKind::F32);
        assert_eq!(
            <Option<u32> as ColumnType>::KIND,
            ColumnKind::Optional(&ColumnKind::U32)
        );
        assert_eq!(
            <Option<Option<String>> as ColumnType>::KIND,
            ColumnKind::Optional(&ColumnKind::Optional(&ColumnKind::String))
        );
    }

    #[test]
    fn cmp_orders_floats_totally_and_none_first() {
        assert_eq!(<f64 as ColumnType>::cmp(&1.0, &f64::NAN), Ordering::Less);
        assert_eq!(
            <Option<i32> as ColumnType>::cmp(&None, &Some(0)),
            Ordering::Less
        );
        assert_eq!(
            <Option<i32> as ColumnType>::cmp(&Some(1), &Some(2)),
            Ordering::Less
        );
    }
}
