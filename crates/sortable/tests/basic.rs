//! Smoke test over the public API surface only.

use sortable::{Table, row};

#[test]
fn typed_rows_via_public_api() {
    let mut table = Table::<(u32, String, i64)>::new(["uid", "user", "ppid"]);

    table.push(row![0u32, "root", 1]);
    table.push(row![1000u32, "alice", 42]);

    assert_eq!(table.len(), 2);
    assert_eq!(table.column_index("user"), Some(1));
    assert_eq!(table.row(1), Some(&(1000, "alice".to_string(), 42)));
    assert_eq!(table.get_as::<u32>(0, 0), Some(&0));
    assert_eq!(table.rows().count(), 2);
    assert_eq!(table.as_slice().len(), 2);
}
