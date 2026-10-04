//! Runtime column projections: borrowed views over a table's columns.

use crate::{ColumnKind, ColumnType, NamedRow, Table};

/// A runtime column selection over a [`Table`], e.g. a user-chosen list of
/// display columns.
///
/// The projection only borrows the table: unselected columns remain fully
/// readable (e.g. for sorting). `columns[i]` is the table index of the
/// projection's i-th column, resolved once at construction, so the read path
/// does no name lookups.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "derive")] {
/// use sortable::{NamedRow, Table};
///
/// #[derive(NamedRow)]
/// struct Process {
///     pid: u32,
///     user: String,
/// }
///
/// let mut table = Table::<Process>::default();
/// table.push(Process { pid: 3, user: "root".to_string() });
///
/// // Projection order is display order, independent of table order.
/// let projection = table.project(["user", "pid"]).unwrap();
/// assert_eq!(projection.names().collect::<Vec<_>>(), ["user", "pid"]);
/// assert_eq!(
///     projection.get_as::<String>(0, 0).map(String::as_str),
///     Some("root"),
/// );
/// # }
/// ```
#[derive(Debug)]
pub struct Projection<'a, R: NamedRow> {
    table: &'a Table<R>,
    columns: Vec<usize>,
}

impl<'a, R: NamedRow> Projection<'a, R> {
    pub(crate) fn new(table: &'a Table<R>, columns: Vec<usize>) -> Self {
        debug_assert!(columns.iter().all(|&i| i < table.column_len()));
        Self { table, columns }
    }

    /// The selected headers, in projection order, borrowed from the table's
    /// names.
    pub fn names(&self) -> impl ExactSizeIterator<Item = &'a str> {
        self.columns.iter().map(|&i| self.table.names()[i].as_str())
    }

    /// The selected column kinds, in projection order.
    pub fn kinds(&self) -> impl ExactSizeIterator<Item = &'static ColumnKind> {
        self.columns.iter().map(|&i| &R::KINDS[i])
    }

    /// Borrows the cell at (`row`, projection-local `column`) as `T`, or
    /// `None` if either index is out of bounds or the cell's kind does not
    /// match `T`.
    pub fn get_as<T: ColumnType>(&self, row: usize, column: usize) -> Option<&T> {
        self.table.get_as(row, *self.columns.get(column)?)
    }

    /// Number of rows: a projection does not filter rows.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Whether the table has no rows.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Number of selected columns.
    pub fn column_len(&self) -> usize {
        self.columns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    struct Process {
        pid: u32,
        user: String,
    }

    impl NamedRow for Process {
        const KINDS: &'static [ColumnKind] = &[ColumnKind::U32, ColumnKind::String];
        const NAMES: &'static [&'static str] = &["pid", "user"];

        fn cell_as<T: ColumnType>(&self, index: usize) -> Option<&T> {
            match index {
                0 => (&self.pid as &dyn Any).downcast_ref(),
                1 => (&self.user as &dyn Any).downcast_ref(),
                _ => None,
            }
        }
    }

    fn table() -> Table<Process> {
        let mut table = Table::default();
        table.push(Process {
            pid: 3,
            user: "root".to_string(),
        });
        table.push(Process {
            pid: 2663,
            user: "alice".to_string(),
        });
        table
    }

    #[test]
    fn translates_projection_coordinates_to_table_coordinates() {
        let table = table();
        let projection = table.project(["user", "pid"]).unwrap(); // display order ≠ table order

        assert_eq!(projection.names().collect::<Vec<_>>(), ["user", "pid"]);
        assert_eq!(
            projection.kinds().collect::<Vec<_>>(),
            [&ColumnKind::String, &ColumnKind::U32]
        );
        assert_eq!(projection.column_len(), 2);
        assert_eq!(projection.len(), 2);

        // Projection-local (1, 0) is table (1, 1): alice's user.
        assert_eq!(
            projection.get_as::<String>(1, 0).map(String::as_str),
            Some("alice")
        );
        // Projection-local (1, 1) is table (1, 0): alice's pid.
        assert_eq!(projection.get_as::<u32>(1, 1), Some(&2663));
        // Out of bounds in projection space.
        assert!(projection.get_as::<u32>(0, 2).is_none());
        // Kind mismatch.
        assert!(projection.get_as::<u32>(0, 0).is_none());
    }

    #[test]
    fn rejects_unknown_column_names() {
        assert!(table().project(["pid", "nope"]).is_none());
    }
}
