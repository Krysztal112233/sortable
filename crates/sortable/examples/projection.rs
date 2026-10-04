//! `derive_table` plus column projection: runtime-selected display columns.
//!
//! Run with: cargo run -p sortable --example projection

use sortable::{NamedRow, Table, row};

#[derive(NamedRow)]
struct Process {
    uid: u32,
    user: String,
    ppid: i64,
    pid: u32,
    tty: Option<String>,
    stat: String,
    comm: String,
}

fn main() {
    let mut table = Table::<Process>::default();

    #[rustfmt::skip]
    vec![
        row![0u32,    "root",  2,    3u32,    None,          "S",    "kthreadd"],
        row![1000u32, "alice", 2655, 2663u32, Some("pts/0"), "Ss",   "fish"],
        row![0u32,    "root",  1290, 1857u32, Some("tty1"),  "S",    "sddm-helper"],
        row![1000u32, "alice", 1290, 1339u32, Some("tty2"),  "Ssl+", "Xorg"],
    ]
    .into_iter()
    .for_each(|it| table.push(it));

    // `ps -o pid,user,comm`: display a subset, in display order. An unknown
    // column name fails fast here, before any output.
    let projection = table.project(["pid", "user", "comm"]).unwrap();

    // Header from the projection itself; casing is display-side.
    for name in projection.names() {
        print!("{:>10} ", name.to_uppercase());
    }
    println!();

    // Cells: the caller knows each selected column's type statically; the
    // projection maps each position to its table column.
    for row in 0..projection.len() {
        let pid = projection.get_as::<u32>(row, 0).unwrap();
        let user = projection.get_as::<String>(row, 1).unwrap();
        let comm = projection.get_as::<String>(row, 2).unwrap();
        println!("{pid:>10} {user:>10} {comm:>10}");
    }

    // The projection only borrows the table: unselected columns remain
    // fully reachable (e.g. ppid for a future `--sort=ppid`).
    assert_eq!(table.get_as::<i64>(0, 2), Some(&2));
}
