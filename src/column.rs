//! Column-level type machinery: kinds and their mapping from Rust types.

use std::fmt;

/// Type tag of a column: which Rust type its cells hold. Derived from the
/// row type at compile time (see [`Row::KINDS`](crate::Row::KINDS)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum ColumnKind {
    F64, F32,
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
/// All implementors are owned, `'static`, cloneable types.
pub trait ColumnType: Sized + 'static + Clone {
    const KIND: ColumnKind;
}

macro_rules! impl_column_type {
   ($($t:ty => $variant:ident),* $(,)?) => {$(
       impl ColumnType for $t {
           const KIND: ColumnKind = ColumnKind::$variant;
       }
   )*};
}

impl_column_type! {
    f64 => F64, f32 => F32,
    i128 => I128, i16 => I16, i32 => I32, i64 => I64, i8 => I8, isize => ISize,
    u128 => U128, u16 => U16, u32 => U32, u64 => U64, u8 => U8, usize => USize,
    bool => Bool,
    String => String,
}

/// `Option<T>` cells are nullable; the kind wraps the inner kind.
impl<T: ColumnType> ColumnType for Option<T> {
    const KIND: ColumnKind = ColumnKind::Optional(&T::KIND);
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
}
