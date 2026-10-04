//! `derive_table` plus multi-key sorting: rows are reordered in place by a
//! `(column name, direction)` key list.
//!
//! Run with: cargo run -p sortable --example sort

use sortable::{NamedRow, SortDirection, Table, row};

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

fn print_processes(title: &str, table: &Table<Process>) {
    println!("{title}");
    for process in table.rows() {
        let Process {
            uid,
            user,
            ppid,
            pid,
            tty,
            stat,
            comm,
        } = process;
        let tty = tty.as_deref().unwrap_or("?");
        println!("{uid:>5} {user:<8} {ppid:>5} {pid:>7} {tty:<6} {stat:<5} {comm}");
    }
    println!();
}

fn main() {
    let mut table = Table::<Process>::default();

    #[rustfmt::skip]
    vec![
        row![0u32,    "root",  1290, 1857u32, Some("tty1"),  "S",    "sddm-helper"],
        row![1000u32, "alice", 2655, 2663u32, Some("pts/0"), "Ss",   "fish"],
        row![0u32,    "root",  2,    3u32,    None,          "S",    "kthreadd"],
        row![1000u32, "alice", 1290, 1339u32, Some("tty2"),  "Ssl+", "Xorg"],
    ]
    .into_iter()
    .for_each(|it| table.push(it));

    print_processes("as pushed:", &table);

    // Multi-key: uid ascending, pid descending within each uid. An unknown
    // column name returns None before any row moves.
    table
        .sort([
            ("uid", SortDirection::Ascending),
            ("pid", SortDirection::Descending),
        ])
        .unwrap();
    print_processes("sorted by uid asc, pid desc:", &table);
}
