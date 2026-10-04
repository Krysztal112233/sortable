//! A tour of `sortable`'s typed table data model, using a `ps`-like
//! process table (the crate's motivating use case).
//!
//! The row schema is implemented **by hand** here to show the principle:
//! everything below the struct is exactly what `#[derive(NamedRow)]` would
//! generate for you — the `NamedRow` impl (column kinds, headers, typed
//! cell access) and the `From` impl bridging a `row!` tuple into the struct.
//! For the concise derived version, see `derive_table.rs`.
//!
//! Run with: cargo run -p sortable --example simple_table

use sortable::{ColumnKind, ColumnType, NamedRow, Table, row};
use std::any::Any;
use std::cmp::Ordering;

/// The row type is a plain struct. It is the input *and* output type of the
/// whole pipeline: build a table from process data, sort it (a later
/// milestone), and get the same structs back in a different order.
/// `Option<String>` makes TTY nullable: kernel threads have no controlling
/// terminal, exactly like `ps` prints `?` there.
#[derive(Clone, Debug, PartialEq)]
struct Process {
    uid: u32,
    user: String,
    ppid: i64,
    pid: u32,
    tty: Option<String>,
    stat: String,
    comm: String,
}

// === What `#[derive(NamedRow)]` generates ===

impl NamedRow for Process {
    // Column kinds, one per field, via `<FieldType as ColumnType>::KIND`.
    const KINDS: &'static [ColumnKind] = &[
        ColumnKind::U32,
        ColumnKind::String,
        ColumnKind::I64,
        ColumnKind::U32,
        ColumnKind::Optional(&ColumnKind::String),
        ColumnKind::String,
        ColumnKind::String,
    ];

    // The headers: field names, in order.
    const NAMES: &'static [&'static str] = &["uid", "user", "ppid", "pid", "tty", "stat", "comm"];

    // Typed cell access by position.
    fn cell_as<T: ColumnType>(&self, index: usize) -> Option<&T> {
        match index {
            0 => (&self.uid as &dyn Any).downcast_ref(),
            1 => (&self.user as &dyn Any).downcast_ref(),
            2 => (&self.ppid as &dyn Any).downcast_ref(),
            3 => (&self.pid as &dyn Any).downcast_ref(),
            4 => (&self.tty as &dyn Any).downcast_ref(),
            5 => (&self.stat as &dyn Any).downcast_ref(),
            6 => (&self.comm as &dyn Any).downcast_ref(),
            _ => None,
        }
    }

    // Per-column comparison via the column type's total order.
    fn cmp_cell(&self, other: &Self, index: usize) -> Option<Ordering> {
        match index {
            0 => Some(ColumnType::cmp(&self.uid, &other.uid)),
            1 => Some(ColumnType::cmp(&self.user, &other.user)),
            2 => Some(ColumnType::cmp(&self.ppid, &other.ppid)),
            3 => Some(ColumnType::cmp(&self.pid, &other.pid)),
            4 => Some(ColumnType::cmp(&self.tty, &other.tty)),
            5 => Some(ColumnType::cmp(&self.stat, &other.stat)),
            6 => Some(ColumnType::cmp(&self.comm, &other.comm)),
            _ => None,
        }
    }
}

/// Bridges the `row!` tuple into the struct, in field order.
impl From<(u32, String, i64, u32, Option<String>, String, String)> for Process {
    fn from(row: (u32, String, i64, u32, Option<String>, String, String)) -> Self {
        Self {
            uid: row.0,
            user: row.1,
            ppid: row.2,
            pid: row.3,
            tty: row.4,
            stat: row.5,
            comm: row.6,
        }
    }
}

// === Hand-written schema ends; usage begins ===

fn main() {
    // Empty table; the headers come from `Process::NAMES` at compile time.
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

    // Schema metadata: the names are the field names (display-side casing,
    // e.g. "uid" vs "UID", is the consumer's choice).
    println!(
        "schema ({} columns, {} rows):",
        table.column_len(),
        table.len()
    );
    for (name, kind) in table.names().iter().zip(table.kinds()) {
        println!("  {:7} {}", name, kind);
    }
    println!();

    // Borrowed, typed reads: zero cloning, even for the `String` cells.
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

    // Single-cell access, statically typed all the way down.
    let tty_col = table.column_index("tty").unwrap();
    let fish_tty = table.get_as::<Option<String>>(1, tty_col).unwrap();
    println!("\ncell (1, tty): {fish_tty:?}");
    assert_eq!(fish_tty.as_deref(), Some("pts/0"));

    // The procps round trip: after a (future) sort, what comes back out is
    // just a Vec of the original structs, in a different order.
    let processes: Vec<Process> = table.into_rows();
    println!("got back {} Process values", processes.len());

    // Sorting arrives in a later milestone — watch this space.
}
