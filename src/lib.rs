//! sortable — a typed, row-oriented table (sorting arrives in a later milestone).
//!
//! Current scope: the table data model. A row type is a tuple of
//! [`ColumnType`]s (see [`Row`]); the column kinds are derived from it at
//! compile time and [`Table`] stores rows natively as `R`, so everything is
//! statically typed — pushes cannot fail and reads are borrowed. Sorting
//! (sort-spec parsing, comparators) is intentionally not implemented yet.

mod column;
mod row;
mod table;

pub use column::{ColumnKind, ColumnType};
pub use row::Row;
pub use table::Table;
