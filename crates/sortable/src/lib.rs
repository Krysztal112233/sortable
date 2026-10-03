//! sortable — a typed, row-oriented table (sorting arrives in a later milestone).
//!
//! A row type is a struct with named fields; deriving [`NamedRow`] (feature
//! `derive`) generates its table schema — the column kinds and the column
//! names — from the fields. [`Table`] stores rows natively as `R`, so
//! everything is statically typed: pushes cannot fail and reads are
//! borrowed. Sorting (sort-spec parsing, comparators) is intentionally not
//! implemented yet.

mod column;
mod row;
mod table;

pub use column::{ColumnKind, ColumnType};
pub use row::NamedRow;
pub use table::Table;

#[cfg(feature = "derive")]
pub use sortable_derive::NamedRow;
