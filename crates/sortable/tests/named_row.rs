//! End-to-end test of the `NamedRow` derive (requires the `derive` feature).
#![cfg(feature = "derive")]

use sortable::{ColumnKind, NamedRow, Table, row};

#[derive(NamedRow)]
struct Process {
    uid: u32,
    user: String,
    tty: Option<String>,
}

#[test]
fn derived_named_row_builds_table_without_names() {
    let mut table = Table::<Process>::default();
    table.push(Process {
        uid: 3,
        user: "root".to_string(),
        tty: None,
    });
    // The generated `From` impl bridges a `row!` tuple into the struct.
    table.push(row![2663u32, "alice", Some("pts/0")]);

    assert_eq!(table.names(), ["uid", "user", "tty"]);
    assert_eq!(
        table.kinds(),
        [
            ColumnKind::U32,
            ColumnKind::String,
            ColumnKind::Optional(&ColumnKind::String),
        ]
    );
    assert_eq!(table.row(0).map(|r| r.user.as_str()), Some("root"));
    assert_eq!(table.row(1).map(|r| r.tty.as_deref()), Some(Some("pts/0")));
    assert_eq!(table.get_as::<Option<String>>(0, 2), Some(&None));
    assert_eq!(table.column_index("tty"), Some(2));

    // The procps round trip: rows come back out as plain structs.
    let processes: Vec<Process> = table.into_rows();
    assert_eq!(processes.len(), 2);
    assert_eq!(processes[1].uid, 2663);
}
