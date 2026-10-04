//! Named rows: a struct type defines a table's row schema.

use crate::column::{ColumnKind, ColumnType};
use std::cmp::Ordering;

/// Builds a row as a tuple of plain values.
///
/// The macro expands to a tuple with each value passed through
/// [`Into::into`]. Fed into [`Table::push`](crate::Table::push), the tuple
/// is converted into the table's row struct via the `From` impl that
/// `#[derive(NamedRow)]` generates — so rows can be written positionally:
///
/// ```
/// # use sortable::row;
/// let row = row![0u32, "root", 1];
/// assert_eq!(row, (0u32, "root".to_string(), 1i64));
/// ```
///
/// - `\&str` literals become `String` automatically;
/// - `Some(inner)` literals convert `inner`, so `Some("root")` becomes
///   `Some("root".to_string())`; bare `None` works as-is;
/// - numeric literals follow the usual defaults (`i32`/`f64`), so add a
///   suffix (`0u32`) when the field type is not reachable from the default
///   (`i64` and `i128` are fine bare, since they implement `From<i32>`).
///
/// With a derived row type, the tuple feeds straight into a table:
///
/// ```
/// # #[cfg(feature = "derive")] {
/// # use sortable::{row, NamedRow, Table};
/// #[derive(NamedRow)]
/// struct Process { uid: u32, user: String, ppid: i64 }
///
/// let mut table = Table::<Process>::default();
/// table.push(row![0u32, "root", 1]); // (u32, String, i64) -> Process
/// # }
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

/// A table row with a compile-time schema.
///
/// `NamedRow` ties together everything a [`Table`](crate::Table) needs to
/// know about its row type: the column kinds ([`NamedRow::KINDS`]), the
/// column names — i.e. the headers ([`NamedRow::NAMES`]) — and typed access
/// to cells by position ([`NamedRow::cell_as`]).
///
/// Almost always derived on a struct with named fields (feature `derive`):
/// the field names become the column names and the field types determine the
/// column kinds via [`ColumnType`]. The derive only writes this boilerplate
/// for you; manual implementations work just as well.
pub trait NamedRow: Sized {
    /// Column kinds, in field order.
    const KINDS: &'static [ColumnKind];

    /// Column names (the headers), in field order.
    const NAMES: &'static [&'static str];

    /// Borrows the cell at `index` as `T`, or `None` if the index is out of
    /// bounds or the cell's type does not match `T`.
    fn cell_as<T: ColumnType>(&self, index: usize) -> Option<&T>;

    /// Compares the cell at `index` with `other`'s using the column type's
    /// total order ([`ColumnType::cmp`]), or `None` if the index is out of
    /// bounds.
    fn cmp_cell(&self, other: &Self, index: usize) -> Option<Ordering>;
}
