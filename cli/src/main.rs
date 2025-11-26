use anyhow::{Context, Result};
use clap::Parser;
use keylite_kv::db::Db;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "keylite")]
#[command(about = "KeyLite - Interactive key-value store shell", long_about = None)]
struct Cli {}

fn print_help() {
    println!("KeyLite Interactive Shell");
    println!();
    println!("Commands:");
    println!("  open <path>          Open a database file");
    println!("  put <key> <value>    Put a key-value pair into the database");
    println!("  get <key>            Get a value by key from the database");
    println!("  del <key>            Delete a key from the database");
    println!("  scan [start] [end]   Scan range of keys from start to end");
    println!("  ping                 Ping the active database");
    println!();
    println!("Meta-commands:");
    println!("  .help                Show this help message");
    println!("  .exit                Exit the shell");
    println!("  .quit                Exit the shell");
    println!();
}

fn format_duration(duration: std::time::Duration) -> String {
    let nanos = duration.as_nanos();
    if nanos < 1_000 {
        format!("{}ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.2}μs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", nanos as f64 / 1_000_000_000.0)
    }
}

enum CommandResult {
    Continue,
    Exit,
}

fn execute_command(db: &mut Option<(String, Db)>, line: &str) -> Result<CommandResult> {
    let line = line.trim();
    if line.starts_with('.') {
        match line {
            ".help" => {
                print_help();
                return Ok(CommandResult::Continue);
            }
            ".exit" | ".quit" => {
                println!("Goodbye!");
                return Ok(CommandResult::Exit);
            }
            _ => {
                println!("Unknown meta-command: {}", line);
                return Ok(CommandResult::Continue);
            }
        }
    }

    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.is_empty() {
        return Ok(CommandResult::Continue);
    }

    let command = parts[0].to_lowercase();
    let start = Instant::now();

    match command.as_str() {
        "open" => {
            if parts.len() < 2 {
                println!("Error: open requires <path>");
                println!("Usage: open <path>");
                return Ok(CommandResult::Continue);
            }

            let path = parts[1].to_string();
            println!("Opening database: {}", path);

            let open_start = Instant::now();
            let new_db = Db::open(&path).context("Failed to open database")?;
            let open_duration = open_start.elapsed();

            *db = Some((path.clone(), new_db));
            println!("Opened in {}", format_duration(open_duration));

            return Ok(CommandResult::Continue);
        }

        "ping" => {
            match db {
                Some((path, _)) => println!("pong ({})", path),
                None => println!("no database connected"),
            }
            return Ok(CommandResult::Continue);
        }

        "put" => {
            let (_, db_ref) = match db {
                Some(tuple) => tuple,
                None => {
                    println!("Error: no database connected");
                    println!("Use: open <path>");
                    return Ok(CommandResult::Continue);
                }
            };

            if parts.len() < 3 {
                println!("Error: put requires <key> <value>");
                return Ok(CommandResult::Continue);
            }

            db_ref
                .put(parts[1].as_bytes(), parts[2].as_bytes())
                .context("Failed to put value")?;

            println!("OK ({})", format_duration(start.elapsed()));
        }

        "get" => {
            let (_, db_ref) = match db {
                Some(tuple) => tuple,
                None => {
                    println!("Error: no database connected");
                    return Ok(CommandResult::Continue);
                }
            };

            if parts.len() < 2 {
                println!("Error: get requires <key>");
                return Ok(CommandResult::Continue);
            }

            match db_ref.get(parts[1].as_bytes()) {
                Ok(val) => match val {
                    Some(value) => {
                        match String::from_utf8(value.clone()) {
                            Ok(s) => println!("{}", s),
                            Err(_) => println!("(binary data)"),
                        }
                        println!("({})", format_duration(start.elapsed()));
                    }
                    None => {
                        println!("(nil)");
                        println!("({})", format_duration(start.elapsed()));
                    }
                },
                Err(e) => {
                    println!("Error: {:?}", e.to_string());
                }
            }
        }

        "del" => {
            let (path, db_ref) = match db {
                Some(tuple) => tuple,
                None => {
                    println!("Error: no database connected");
                    return Ok(CommandResult::Continue);
                }
            };

            if parts.len() < 2 {
                println!("Error: del requires <key>");
                return Ok(CommandResult::Continue);
            }

            db_ref
                .del(parts[1].as_bytes())
                .context("Failed to delete key")?;

            println!("OK ({}) [{}]", format_duration(start.elapsed()), path);
        }

        "scan" => {
            let (_, db_ref) = match db {
                Some(tup) => tup,
                None => {
                    println!("Error: no database connected");
                    return Ok(CommandResult::Continue);
                }
            };
            let start_bound: Option<&[u8]> = if parts.len() > 1 {
                Some(&parts[1].as_bytes())
            } else {
                None
            };

            let end_bound: Option<&[u8]> = if parts.len() > 2 {
                Some(&parts[2].as_bytes())
            } else {
                None
            };

            let start = Instant::now();
            let results: Vec<_> = db_ref.scan(start_bound, end_bound).collect();

            for (key, value) in results.into_iter() {
                println!(
                    "key: {:?} value: {:?}",
                    String::from_utf8(key).unwrap_or_else(|_| "nil".to_string()),
                    String::from_utf8(value).unwrap_or_else(|_| "nil".to_string())
                );
            }
            println!("({})", format_duration(start.elapsed()));
        }

        _ => println!("Unknown command: {}", command),
    }

    Ok(CommandResult::Continue)
}

fn main() -> Result<()> {
    let _cli = Cli::parse();

    let mut db: Option<(String, Db)> = None;

    println!("KeyLite version 0.1.0");
    println!("No database connected.");
    println!("Use: open <path>");
    println!("Type .help for usage hints");
    println!();

    let mut rl = DefaultEditor::new()?;
    let history_path = format!("/var/lib/keylite/.keylite_history");
    let _ = rl.load_history(&history_path);

    loop {
        match rl.readline("keylite> ") {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(&line);

                match execute_command(&mut db, &line) {
                    Ok(CommandResult::Continue) => {}
                    Ok(CommandResult::Exit) => break,
                    Err(e) => println!("Error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => println!("^C"),
            Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_path);
    Ok(())
}
