//! The same `ps`-like table as `simple_table.rs`, but with the row schema
//! derived instead of hand-written — this file is the entire definition.
//! For runtime column selection, see `projection.rs`.
//!
//! Run with: cargo run -p sortable --example derive_table

use sortable::{NamedRow, Table, row};

/// `NamedRow` derives the column kinds, the headers (field names), typed
/// cell access, and the `From` impl that lets `row!` tuples feed `push`.
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

    println!(
        "{:>5} {:<8} {:>5} {:>5} {:<6} {:<5} COMMAND",
        "UID", "USER", "PPID", "PID", "TTY", "STAT"
    );
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
        println!("{uid:>5} {user:<8} {ppid:>5} {pid:>5} {tty:<6} {stat:<5} {comm}");
    }
}
