//! Sorting: multi-key table sorting with per-key direction.

use crate::row::NamedRow;
use crate::table::Table;
use std::cmp::Ordering;

/// Direction of a single sort key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn apply(self, ordering: Ordering) -> Ordering {
        match self {
            Self::Ascending => ordering,
            Self::Descending => ordering.reverse(),
        }
    }
}

impl<R: NamedRow> Table<R> {
    /// Sorts rows by the given `(column name, direction)` keys: the first
    /// key that distinguishes two rows decides. The sort is stable — rows
    /// equal on all keys keep their relative order. Returns `None` if a key
    /// names an unknown column; the table is then left unmodified.
    pub fn sort(
        &mut self,
        keys: impl IntoIterator<Item = (impl AsRef<str>, SortDirection)>,
    ) -> Option<()> {
        let keys: Vec<(usize, SortDirection)> = keys
            .into_iter()
            .map(|(name, direction)| self.column_index(name.as_ref()).map(|i| (i, direction)))
            .collect::<Option<_>>()?;
        self.rows.sort_by(|a, b| {
            keys.iter()
                .filter_map(|&(i, direction)| a.cmp_cell(b, i).map(|o| direction.apply(o)))
                .find(|&o| o != Ordering::Equal)
                .unwrap_or(Ordering::Equal)
        });
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{ColumnKind, ColumnType};

    // Hand-written `NamedRow` impl: core tests run without the `derive`
    // feature (see the table.rs tests for the same pattern).
    #[derive(Debug, PartialEq)]
    struct Process {
        uid: u32,
        tty: Option<String>,
    }

    impl NamedRow for Process {
        const KINDS: &'static [ColumnKind] =
            &[ColumnKind::U32, ColumnKind::Optional(&ColumnKind::String)];
        const NAMES: &'static [&'static str] = &["uid", "tty"];

        fn cell_as<T: ColumnType>(&self, index: usize) -> Option<&T> {
            match index {
                0 => (&self.uid as &dyn std::any::Any).downcast_ref(),
                1 => (&self.tty as &dyn std::any::Any).downcast_ref(),
                _ => None,
            }
        }

        fn cmp_cell(&self, other: &Self, index: usize) -> Option<Ordering> {
            match index {
                0 => Some(ColumnType::cmp(&self.uid, &other.uid)),
                1 => Some(ColumnType::cmp(&self.tty, &other.tty)),
                _ => None,
            }
        }
    }

    #[test]
    fn sorts_by_multiple_keys_with_directions() {
        let mut table = Table::from(vec![
            Process {
                uid: 2,
                tty: Some("b".to_string()),
            },
            Process { uid: 1, tty: None },
            Process {
                uid: 2,
                tty: Some("a".to_string()),
            },
            Process {
                uid: 1,
                tty: Some("z".to_string()),
            },
        ]);

        use SortDirection::{Ascending as A, Descending as D};
        table.sort([("uid", A), ("tty", A)]).unwrap();
        let uids: Vec<u32> = table.rows().map(|p| p.uid).collect();
        assert_eq!(uids, [1, 1, 2, 2]);
        // None sorts first within the uid = 1 group.
        assert!(table.row(0).unwrap().tty.is_none());
        // Descending flips the key.
        table.sort([("uid", D)]).unwrap();
        assert_eq!(table.row(0).map(|p| p.uid), Some(2));
    }

    #[test]
    fn unknown_key_leaves_table_unmodified() {
        let mut table = Table::from(vec![
            Process { uid: 2, tty: None },
            Process { uid: 1, tty: None },
        ]);
        assert!(table.sort([("nope", SortDirection::Ascending)]).is_none());
        assert_eq!(table.row(0).map(|p| p.uid), Some(2));
    }
}
