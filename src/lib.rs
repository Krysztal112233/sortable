//! sortable — a typed, row-oriented table (sorting arrives in a later milestone).

mod column;
mod row;

pub use column::{ColumnKind, ColumnType};
pub use row::Row;
