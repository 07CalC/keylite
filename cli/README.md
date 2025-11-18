# KeyLite CLI

An interactive shell for KeyLite, a high-performance LSM-Tree based key-value store. Similar to the SQLite shell, this provides a convenient way to interact with your KeyLite database.

## Installation

Build from source:

```bash
cargo build --release --package keylite-cli
```

The binary will be available at `./target/release/keylite-cli`.

## Usage

### Interactive Shell (Default)

Start the interactive shell:

```bash
keylite-cli --path ./mydb
```

This will open an interactive prompt:

```
KeyLite version 0.1.0
Database: ./mydb
Opened in 45.23μs
Type .help for usage hints

keylite> 
```

### Commands

#### Put a key-value pair

```
keylite> put username john_doe
OK (12.34μs)

keylite> put email john@example.com
OK (8.91μs)
```

#### Get a value by key

```
keylite> get username
john_doe
(15.67μs | 1 row)

keylite> get nonexistent
(nil)
(23.45μs | 0 rows)
```

#### Delete a key

```
keylite> del username
OK (10.23μs)
```

### Meta-commands

- `.help` - Show help message
- `.exit` or `.quit` - Exit the shell
- `Ctrl-D` - Exit the shell
- `Ctrl-C` - Cancel current input

### Features

- **Interactive REPL** - Similar to SQLite, Redis, and other database shells
- **Command history** - Use arrow keys to navigate previous commands (saved to `.keylite_history`)
- **Performance timing** - Every operation shows execution time
- **Human-readable durations** - Automatically formats as ns, μs, ms, or s
- **UTF-8 support** - Handles text data with automatic encoding
- **Binary data detection** - Safely handles non-UTF8 values

## Examples

### Complete Session

```
$ keylite-cli --path ./mydb
KeyLite version 0.1.0
Database: ./mydb
Opened in 45.23μs
Type .help for usage hints

keylite> put name Alice
OK (12.34μs)

keylite> put age 30
OK (8.91μs)

keylite> put city "New York"
OK (10.45μs)

keylite> get name
Alice
(15.67μs | 1 row)

keylite> get age
30
(18.23μs | 1 row)

keylite> del age
OK (10.23μs)

keylite> get age
(nil)
(23.45μs | 0 rows)

keylite> .exit
Goodbye!
```

### Performance Monitoring

The shell shows execution time for every operation:
- `ns` - Nanoseconds (< 1μs)
- `μs` - Microseconds (< 1ms)
- `ms` - Milliseconds (< 1s)
- `s` - Seconds (≥ 1s)

This makes it easy to monitor database performance and identify slow operations.

## Command Line Options

```
Options:
  -p, --path <PATH>  Path to the database directory [default: ./data]
  -h, --help         Print help
```

## History

Command history is automatically saved to `.keylite_history` in your database directory. This allows you to use the up/down arrow keys to navigate through previous commands across sessions.
