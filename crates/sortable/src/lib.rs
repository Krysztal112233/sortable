//! sortable — a typed, row-oriented table.
//!
//! A row type is a struct with named fields; deriving [`NamedRow`] (feature
//! `derive`) generates its table schema — the column kinds and the column
//! names — from the fields. [`Table`] stores rows natively as `R`, so
//! everything is statically typed: pushes cannot fail and reads are
//! borrowed. [`Projection`] borrows a table with a runtime-selected column
//! list; [`Table::sort`] orders rows by a `(name, `[`SortDirection`]`)` key
//! list.

mod column;
mod projection;
mod row;
mod sort;
mod table;

pub use column::{ColumnKind, ColumnType};
pub use projection::Projection;
pub use row::NamedRow;
pub use sort::SortDirection;
pub use table::Table;

#[cfg(feature = "derive")]
pub use sortable_derive::NamedRow;
