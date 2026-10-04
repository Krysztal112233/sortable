//! The public [`Table`] type: natively stored, schema-typed rows.

use crate::column::{ColumnKind, ColumnType};
use crate::projection::Projection;
use crate::row::NamedRow;

/// A typed, row-oriented table whose row type carries a compile-time schema
/// (see [`NamedRow`]).
///
/// Rows are stored natively as `R`, so typed reads are borrowed and
/// clone-free, pushes cannot fail, and there is no runtime validation.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "derive")] {
/// use sortable::{NamedRow, Table};
///
/// #[derive(NamedRow)]
/// struct Process {
///     uid: u32,
///     user: String,
///     ppid: i64,
/// }
///
/// // Empty table; headers are the field names: "uid", "user", "ppid".
/// let mut table = Table::<Process>::default();
/// table.push(Process {
///     uid: 0,
///     user: "root".to_string(),
///     ppid: 1,
/// });
///
/// assert_eq!(table.row(0).map(|r| r.user.as_str()), Some("root"));
/// assert_eq!(table.get_as::<u32>(0, 0), Some(&0));
/// # }
/// ```
#[derive(Debug)]
pub struct Table<R: NamedRow> {
    names: Vec<String>,
    pub(crate) rows: Vec<R>,
}

impl<R: NamedRow> Table<R> {
    /// Creates an empty table with custom column names, overriding
    /// [`NamedRow::NAMES`] (e.g. user-chosen headers).
    ///
    /// # Panics
    ///
    /// Panics if the number of names differs from the row's arity.
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let names: Vec<String> = names.into_iter().map(Into::into).collect();
        assert_eq!(
            names.len(),
            R::NAMES.len(),
            "column name count ({}) must match row arity ({})",
            names.len(),
            R::NAMES.len()
        );
        Self {
            names,
            rows: Vec::new(),
        }
    }

    /// Appends a row.
    ///
    /// Accepts anything convertible into `R`: an `R` itself, or the tuple of
    /// field values built by [`row!`](crate::row) — `#[derive(NamedRow)]`
    /// generates the `From` impl bridging the two.
    pub fn push(&mut self, row: impl Into<R>) {
        self.rows.push(row.into());
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

    /// Consumes the table and returns the rows as a `Vec<R>` — e.g. after
    /// sorting, the same process list in a different order.
    pub fn into_rows(self) -> Vec<R> {
        self.rows
    }

    /// Borrows a single cell as its Rust type.
    ///
    /// Returns `None` if either index is out of bounds or the cell's kind
    /// does not match `T`.
    pub fn get_as<T: ColumnType>(&self, row: usize, column: usize) -> Option<&T> {
        self.rows.get(row)?.cell_as(column)
    }

    /// Selects columns by header name for display or further runtime
    /// reading, resolving each name once. Returns `None` if any name is
    /// unknown.
    ///
    /// The returned [`Projection`] only borrows this table; see its docs.
    pub fn project(
        &self,
        names: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Option<Projection<'_, R>> {
        let columns = names
            .into_iter()
            .map(|name| self.column_index(name.as_ref()))
            .collect::<Option<Vec<_>>>()?;
        Some(Projection::new(self, columns))
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

    /// Number of columns (the row's arity).
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

impl<R: NamedRow> Default for Table<R> {
    /// An empty table using `R`'s compile-time column names
    /// ([`NamedRow::NAMES`]).
    fn default() -> Self {
        Self::new(R::NAMES.iter().copied())
    }
}

impl<R: NamedRow> From<Vec<R>> for Table<R> {
    /// A table pre-filled with `rows`, using `R`'s compile-time column names.
    fn from(rows: Vec<R>) -> Self {
        Self {
            names: R::NAMES.iter().map(|&n| n.to_string()).collect(),
            rows,
        }
    }
}

impl<R: NamedRow> FromIterator<R> for Table<R> {
    /// Collects rows into a table using `R`'s compile-time column names.
    fn from_iter<I: IntoIterator<Item = R>>(iter: I) -> Self {
        let mut table = Self::default();
        table.extend(iter);
        table
    }
}

impl<R: NamedRow> Extend<R> for Table<R> {
    fn extend<I: IntoIterator<Item = R>>(&mut self, iter: I) {
        self.rows.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hand-written `NamedRow` impls: the derive only generates this
    // boilerplate, so the core is tested here without the `derive` feature.
    #[derive(Debug, PartialEq)]
    struct Process {
        uid: u32,
        user: String,
        ppid: i64,
    }

    impl NamedRow for Process {
        const KINDS: &'static [ColumnKind] =
            &[ColumnKind::U32, ColumnKind::String, ColumnKind::I64];
        const NAMES: &'static [&'static str] = &["uid", "user", "ppid"];

        fn cell_as<T: ColumnType>(&self, index: usize) -> Option<&T> {
            match index {
                0 => (&self.uid as &dyn std::any::Any).downcast_ref(),
                1 => (&self.user as &dyn std::any::Any).downcast_ref(),
                2 => (&self.ppid as &dyn std::any::Any).downcast_ref(),
                _ => None,
            }
        }

        fn cmp_cell(&self, other: &Self, index: usize) -> Option<std::cmp::Ordering> {
            match index {
                0 => Some(ColumnType::cmp(&self.uid, &other.uid)),
                1 => Some(ColumnType::cmp(&self.user, &other.user)),
                2 => Some(ColumnType::cmp(&self.ppid, &other.ppid)),
                _ => None,
            }
        }
    }

    fn root() -> Process {
        Process {
            uid: 0,
            user: "root".to_string(),
            ppid: 1,
        }
    }

    #[test]
    fn push_and_read_back_typed() {
        let mut t = Table::<Process>::default();
        t.push(root());

        assert_eq!(t.len(), 1);
        assert!(!t.is_empty());
        assert_eq!(t.row(0), Some(&root()));
        assert_eq!(t.get_as::<u32>(0, 0), Some(&0));
        assert_eq!(t.get_as::<String>(0, 1).map(String::as_str), Some("root"));
        assert_eq!(t.as_slice(), &[root()]);
    }

    #[test]
    fn new_with_custom_names_panics_on_arity_mismatch() {
        let result = std::panic::catch_unwind(|| {
            let _t = Table::<Process>::new(["only-one"]);
        });
        assert!(result.is_err());
    }

    #[test]
    fn access_out_of_bounds_or_wrong_kind_is_none() {
        let mut t = Table::<Process>::default();
        t.push(root());

        assert_eq!(t.get_as::<i32>(0, 0), None); // u32 cell, not i32
        assert_eq!(t.get_as::<u32>(1, 0), None); // no such row
        assert_eq!(t.get_as::<u32>(0, 3), None); // no such column
        assert_eq!(t.row(1), None);
    }

    #[test]
    fn rows_iterate_in_insertion_order() {
        let mut t = Table::<Process>::default();
        t.push(root());
        t.push(Process {
            uid: 1000,
            user: "alice".to_string(),
            ppid: 42,
        });

        let rows: Vec<_> = t.rows().collect();
        assert_eq!(
            rows,
            vec![
                &root(),
                &Process {
                    uid: 1000,
                    user: "alice".to_string(),
                    ppid: 42,
                },
            ]
        );
        assert_eq!(t.rows().len(), 2);
    }

    #[derive(Debug, PartialEq)]
    struct Tty {
        pid: u32,
        tty: Option<String>,
    }

    impl NamedRow for Tty {
        const KINDS: &'static [ColumnKind] =
            &[ColumnKind::U32, ColumnKind::Optional(&ColumnKind::String)];
        const NAMES: &'static [&'static str] = &["pid", "tty"];

        fn cell_as<T: ColumnType>(&self, index: usize) -> Option<&T> {
            match index {
                0 => (&self.pid as &dyn std::any::Any).downcast_ref(),
                1 => (&self.tty as &dyn std::any::Any).downcast_ref(),
                _ => None,
            }
        }

        fn cmp_cell(&self, other: &Self, index: usize) -> Option<std::cmp::Ordering> {
            match index {
                0 => Some(ColumnType::cmp(&self.pid, &other.pid)),
                1 => Some(ColumnType::cmp(&self.tty, &other.tty)),
                _ => None,
            }
        }
    }

    #[test]
    fn option_columns_round_trip() {
        let mut t = Table::<Tty>::default();
        t.push(Tty { pid: 3, tty: None });
        t.push(Tty {
            pid: 2663,
            tty: Some("pts/0".to_string()),
        });

        assert_eq!(t.row(0), Some(&Tty { pid: 3, tty: None }));
        assert_eq!(
            t.get_as::<Option<String>>(1, 1),
            Some(&Some("pts/0".to_string()))
        );
        assert_eq!(t.get_as::<String>(1, 1), None); // the cell IS optional
        assert_eq!(t.kinds()[1], ColumnKind::Optional(&ColumnKind::String));
    }

    #[test]
    fn column_lookup_by_name() {
        let t = Table::<Process>::default();
        assert_eq!(t.column_index("user"), Some(1));
        assert_eq!(t.column_index("nope"), None);
        assert_eq!(t.names()[2], "ppid");
        assert_eq!(t.kinds()[2], ColumnKind::I64);
        assert_eq!(t.column_len(), 3);
    }

    #[test]
    fn from_and_collect_prefill_rows() {
        let rows = vec![root(), root()];
        let t = Table::from(rows);
        assert_eq!(t.names(), ["uid", "user", "ppid"]);
        assert_eq!(t.len(), 2);

        let collected: Table<Process> = vec![root()].into_iter().collect();
        assert_eq!(collected.row(0), Some(&root()));
    }
}
