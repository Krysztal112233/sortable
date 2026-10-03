//! Typed rows: a tuple type defines a table's row schema.

use crate::column::{ColumnKind, ColumnType};

/// Builds a typed table row from plain values.
///
/// The whole macro expands to a tuple, so the result feeds directly into
/// [`Table::push`](crate::Table::push) with full compile-time checking:
///
/// - each value is passed through [`Into::into`], so `&str` literals become `String` automatically;
/// - `Some(inner)` literals convert `inner` instead, so `Some("root")` becomes
///   `Some("root".to_string())`; bare `None` works as-is;
/// - numeric literals follow the usual defaults (`i32`/`f64`), so add a suffix (`0u32`) when the
///   column type is not reachable from the default (`i64` and `i128` are fine bare, since they
///   implement `From<i32>`).
///
/// # Examples
///
/// ```
/// # use sortable::{row, Table};
/// let mut table = Table::<(u32, String, i64)>::new(["uid", "user", "ppid"]);
///
/// table.push(row![0u32, "root", 1]);
/// table.push(row![1000u32, "alice", 2663]);
///
/// assert_eq!(table.row(0), Some(&(0, "root".to_string(), 1)));
/// ```
///
/// `Option` columns take `Some(...)` or `None` directly:
///
/// ```
/// # use sortable::{row, Table};
/// let mut table = Table::<(u32, Option<String>)>::new(["pid", "tty"]);
///
/// table.push(row![3u32, None]);            // kernel thread: no TTY
/// table.push(row![2663u32, Some("pts/0")]);
///
/// assert_eq!(table.row(1).and_then(|r| r.1.as_deref()), Some("pts/0"));
/// ```
#[macro_export]
macro_rules! row {
    ($($args:tt)*) => {
        $crate::__sortable_row_munch![[] $($args)*]
    };
}

/// Implementation detail of [`row!`]: converts the argument list one
/// element at a time so that leading `Some(...)` literals can be rewritten
/// to convert their inner value.
#[doc(hidden)]
#[macro_export]
macro_rules! __sortable_row_munch {
    // Input exhausted: emit the accumulated tuple.
    ([$($out:tt)*]) => {
        ($($out)*)
    };
    // Leading `Some(...)` argument: convert the inner value.
    ([$($out:tt)*] Some($inner:expr) $(, $($rest:tt)*)?) => {
        $crate::__sortable_row_munch![
            [$($out)* ::core::option::Option::Some(::core::convert::Into::into($inner)),]
            $($($rest)*)?
        ]
    };
    // Anything else: a plain expression, converted whole.
    ([$($out:tt)*] $value:expr $(, $($rest:tt)*)?) => {
        $crate::__sortable_row_munch![
            [$($out)* ::core::convert::Into::into($value),]
            $($($rest)*)?
        ]
    };
}

/// A row of a [`Table`](crate::Table), defined as a tuple of [`ColumnType`]s.
///
/// `(u32, String, i64)` describes a three-column row: the column kinds are
/// derived from the element types at compile time, while the column names
/// are given to [`Table::new`](crate::Table::new).
///
/// Implemented for tuples of 1 to 16 elements.
pub trait Row: Sized {
    /// Column kinds, in field order.
    const KINDS: &'static [ColumnKind];

    /// Borrows the cell at `index` as `T`, or `None` if the index is out of
    /// bounds or the cell's type does not match `T`.
    fn cell_as<T: ColumnType>(&self, index: usize) -> Option<&T>;
}

macro_rules! impl_row {
    ($($name:ident : $idx:tt),+ $(,)?) => {
        impl<$($name: ColumnType),+> Row for ($($name,)+) {
            const KINDS: &'static [ColumnKind] = &[$($name::KIND),+];

            fn cell_as<T: ColumnType>(&self, index: usize) -> Option<&T> {
                $(if index == $idx {
                    return (&self.$idx as &dyn std::any::Any).downcast_ref::<T>();
                })*
                None
            }
        }
    };
}

impl_row!(A: 0);
impl_row!(A: 0, B: 1);
impl_row!(A: 0, B: 1, C: 2);
impl_row!(A: 0, B: 1, C: 2, D: 3);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14);
impl_row!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuple_kinds_in_field_order() {
        assert_eq!(
            <(u32, String, i64)>::KINDS,
            &[ColumnKind::U32, ColumnKind::String, ColumnKind::I64]
        );
        assert_eq!(<(bool,)>::KINDS, &[ColumnKind::Bool]);
        assert_eq!(
            <(Option<u32>,)>::KINDS,
            &[ColumnKind::Optional(&ColumnKind::U32)]
        );
    }

    #[test]
    fn cell_accessors() {
        let row = (1u32, "alice".to_string(), None::<String>);

        assert_eq!(row.cell_as::<u32>(0), Some(&1));
        assert_eq!(row.cell_as::<i32>(0), None); // right index, wrong type
        assert_eq!(row.cell_as::<String>(1).map(String::as_str), Some("alice"));
        assert_eq!(row.cell_as::<Option<String>>(2), Some(&None));
        assert_eq!(row.cell_as::<String>(2), None); // the cell IS optional
        assert_eq!(row.cell_as::<u32>(5), None); // out of bounds
    }
}
