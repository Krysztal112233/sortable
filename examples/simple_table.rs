//! A tour of `sortable`'s typed table data model, using a `ps`-like
//! process table (the crate's motivating use case).
//!
//! Run with: cargo run --example simple_table

use sortable::{Table, row};

/// The row type is a plain tuple: column kinds are derived from it at
/// compile time. Field order matches the column names below. `Option<String>`
/// makes the TTY column nullable — kernel threads have no controlling
/// terminal, exactly like `ps` prints `?` there.
type Process = (u32, String, i64, u32, Option<String>, String, String);

fn main() {
    // Only the column names are runtime information; the kinds come from
    // `Process`. Panics if the name count doesn't match the tuple arity.
    let mut table = Table::<Process>::new(["UID", "USER", "PPID", "PID", "TTY", "STAT", "COMMAND"]);

    // Typed pushes cannot fail: arity and kinds are checked by the compiler.
    // `None` and `Some("...")` fill the optional column directly.
    #[rustfmt::skip]
    vec![
        row![0u32,    "root",  2,    3u32,    None,          "S",    "kthreadd"],
        row![1000u32, "alice", 2655, 2663u32, Some("pts/0"), "Ss",   "fish"],
        row![0u32,    "root",  1290, 1857u32, Some("tty1"),  "S",    "sddm-helper"],
        row![1000u32, "alice", 1290, 1339u32, Some("tty2"),  "Ssl+", "Xorg"],
    ]
    .into_iter()
    .for_each(|it| table.push(it));

    // Schema metadata: names are the table's only runtime schema; kinds are
    // derived from `Process` at compile time. Note how TTY reports an
    // optional kind.
    println!(
        "schema ({} columns, {} rows):",
        table.column_len(),
        table.len()
    );
    for (name, kind) in table.names().iter().zip(table.kinds()) {
        println!("  {:7} {}", name, kind);
    }
    let pid_col = table.column_index("PID").unwrap();
    println!("\nPID is column #{pid_col}\n");

    // Borrowed, typed reads: zero cloning, even for the `String` cells.
    println!(
        "{:>5} {:<8} {:>5} {:>5} {:<6} {:<5} COMMAND",
        "UID", "USER", "PPID", "PID", "TTY", "STAT"
    );
    for row in table.rows() {
        let (uid, user, ppid, pid, tty, stat, command) = row;
        let tty = tty.as_deref().unwrap_or("?");
        println!("{uid:>5} {user:<8} {ppid:>5} {pid:>5} {tty:<6} {stat:<5} {command}");
    }

    // Single-cell access, statically typed all the way down.
    let tty_col = table.column_index("TTY").unwrap();
    let fish_tty = table.get_as::<Option<String>>(1, tty_col).unwrap();
    println!("\ncell (1, TTY): {fish_tty:?}");
    assert_eq!(fish_tty.as_deref(), Some("pts/0"));

    // Sorting arrives in a later milestone — watch this space.
}
