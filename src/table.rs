//! The public [`Table`] type: natively stored, tuple-typed rows.

use crate::column::{ColumnKind, ColumnType};
use crate::row::Row;

/// A typed, row-oriented table whose row type is a tuple.
///
/// `Table<(u32, String, i64)>` stores rows of three columns. The schema's
/// only runtime data is the column names given to [`Table::new`]; the kinds
/// are derived from the tuple's element types at compile time
/// ([`Table::kinds`]). Rows are stored natively as `R`, so typed reads are
/// borrowed and clone-free, pushes cannot fail, and there is no runtime
/// validation.
///
/// # Examples
///
/// ```
/// use sortable::{row, Table};
///
/// let mut table = Table::<(u32, String, i64)>::new(["uid", "user", "ppid"]);
/// table.push(row![0u32, "root", 1]);
///
/// assert_eq!(table.row(0), Some(&(0, "root".to_string(), 1)));
/// assert_eq!(table.get_as::<u32>(0, 0), Some(&0));
/// ```
pub struct Table<R: Row> {
    names: Vec<String>,
    rows: Vec<R>,
}

impl<R: Row> Table<R> {
    /// Creates a table from column names; the kinds come from `R`.
    ///
    /// # Panics
    ///
    /// Panics if the number of names differs from the row tuple's arity.
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let names: Vec<String> = names.into_iter().map(Into::into).collect();
        assert_eq!(
            names.len(),
            R::KINDS.len(),
            "column name count ({}) must match row arity ({})",
            names.len(),
            R::KINDS.len()
        );
        Self {
            names,
            rows: Vec::new(),
        }
    }

    /// Appends a typed row. Cannot fail: the kinds are fixed by `R`.
    pub fn push(&mut self, row: R) {
        self.rows.push(row);
    }

    /// Borrows the row at `index`.
    pub fn row(&self, index: usize) -> Option<&R> {
        self.rows.get(index)
    }

    /// Iterates all rows by reference, in insertion order.
    pub fn rows(&self) -> impl DoubleEndedIterator<Item = &R> + ExactSizeIterator {
        self.rows.iter()
    }

    /// All rows as a contiguous slice.
    pub fn as_slice(&self) -> &[R] {
        &self.rows
    }

    /// Borrows a single cell as its Rust type.
    ///
    /// Returns `None` if either index is out of bounds or the cell's kind
    /// does not match `T`.
    pub fn get_as<T: ColumnType>(&self, row: usize, column: usize) -> Option<&T> {
        self.rows.get(row)?.cell_as(column)
    }

    /// The column names, in order.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// The column kinds, in order, as derived from `R` at compile time.
    pub fn kinds(&self) -> &'static [ColumnKind] {
        R::KINDS
    }

    /// Index of the first column with this name, if any.
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n.as_str() == name)
    }

    /// Number of columns (the row tuple's arity).
    pub fn column_len(&self) -> usize {
        self.names.len()
    }

    /// Number of stored rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::ColumnKind;

    fn sample() -> Table<(u32, String, i64)> {
        Table::new(["uid", "user", "ppid"])
    }

    #[test]
    fn push_and_read_back_typed() {
        let mut t = sample();
        t.push((0, "root".to_string(), 1));

        assert_eq!(t.len(), 1);
        assert!(!t.is_empty());
        assert_eq!(t.row(0), Some(&(0, "root".to_string(), 1)));
        assert_eq!(t.get_as::<u32>(0, 0), Some(&0));
        assert_eq!(t.get_as::<String>(0, 1).map(String::as_str), Some("root"));
        assert_eq!(t.as_slice(), &[(0, "root".to_string(), 1)]);
    }

    #[test]
    fn construction_panics_on_name_arity_mismatch() {
        let result = std::panic::catch_unwind(|| {
            let _t: Table<(u32, String)> = Table::new(["only-one"]);
        });
        assert!(result.is_err());
    }

    #[test]
    fn access_out_of_bounds_or_wrong_kind_is_none() {
        let mut t = sample();
        t.push((0, "root".to_string(), 1));

        assert_eq!(t.get_as::<i32>(0, 0), None); // u32 cell, not i32
        assert_eq!(t.get_as::<u32>(1, 0), None); // no such row
        assert_eq!(t.get_as::<u32>(0, 3), None); // no such column
        assert_eq!(t.row(1), None);
    }

    #[test]
    fn rows_iterate_in_insertion_order() {
        let mut t = sample();
        t.push((0, "root".to_string(), 1));
        t.push((1000, "alice".to_string(), 42));

        let rows: Vec<_> = t.rows().collect();
        assert_eq!(
            rows,
            vec![
                &(0u32, "root".to_string(), 1i64),
                &(1000u32, "alice".to_string(), 42i64),
            ]
        );
        assert_eq!(t.rows().len(), 2);
    }

    #[test]
    fn option_columns_round_trip() {
        let mut t = Table::<(u32, Option<String>)>::new(["pid", "tty"]);
        t.push((3, None));
        t.push((2663, Some("pts/0".to_string())));

        assert_eq!(t.row(0), Some(&(3, None)));
        assert_eq!(
            t.get_as::<Option<String>>(1, 1),
            Some(&Some("pts/0".to_string()))
        );
        assert_eq!(t.get_as::<String>(1, 1), None); // the cell IS optional
        assert_eq!(t.kinds()[1], ColumnKind::Optional(&ColumnKind::String));
    }

    #[test]
    fn column_lookup_by_name() {
        let t = sample();
        assert_eq!(t.column_index("user"), Some(1));
        assert_eq!(t.column_index("nope"), None);
        assert_eq!(t.names()[2], "ppid");
        assert_eq!(t.kinds()[2], ColumnKind::I64);
        assert_eq!(t.column_len(), 3);
    }
}
