# keylite

A lightweight, embedded key-value database written in Rust with support for multiple programming languages via C FFI.

## Features

- **Simple key-value storage** with put/get/delete operations
- **SSTable-based architecture** for fast opens and efficient storage
- **No RAM index required** - uses sparse index with bloom filters
- **Fast database opens** - loads metadata in microseconds (no index rebuild)
- **In-memory memtable** with automatic flush to disk
- **Automatic compaction** - merges SSTables automatically
- **Thread-safe operations**
- **Multi-language support via C FFI** (C, Python, Node.js, Go, Ruby, Java, etc.)
- **String API** - Automatic UTF-8 encoding/decoding for text data

## Architecture

Keylite uses a modern SSTable (Sorted String Table) architecture:

- **Memtable**: Recent writes stored in memory (BTreeMap) up to 4MB
- **SSTables**: Immutable sorted files on disk with:
  - 64KB data blocks with sorted key-value pairs
  - Sparse index for fast lookups (one entry per block)
  - Bloom filters for efficient negative lookups
  - CRC32 checksums for data integrity
- **Automatic flushing**: Memtable flushes to SSTable when size threshold reached
- **Automatic compaction**: Merges SSTables when count exceeds threshold

### Performance Benefits

- **Fast opens**: ~50-100 microseconds (no index rebuild needed)
- **Memory efficient**: Only sparse index + bloom filters in RAM (~16-32MB per SSTable with LRU block cache)
- **Fast lookups**: Binary search on sparse index + bloom filters + LRU cache
- **High throughput**: ~675k writes/sec, ~109k sequential reads/sec, ~103k random reads/sec, ~160k negative lookups/sec

## Usage

### Interactive Shell

KeyLite includes an interactive shell similar to SQLite for easy database interaction:

```bash
# Build the CLI
cargo build --release --package keylite-cli

# Start interactive shell
./target/release/keylite-cli --path ./mydb
```

Example session:
```
KeyLite version 0.1.0
Database: ./mydb
Opened in 45.23μs
Type .help for usage hints

keylite> put username Alice
OK (12.34μs)

keylite> get username
Alice
(15.67μs | 1 row)

keylite> del username
OK (10.23μs)

keylite> .exit
Goodbye!
```

Features:
- Interactive REPL with command history
- Performance timing for every operation
- Tab completion and line editing
- Persistent command history

See [keylite-cli/README.md](keylite-cli/README.md) for full CLI documentation.

### Rust Library

```rust
use keylite::db::Db;

let db = Db::open("mydb")?;
db.put(b"hello", b"world")?;
let value = db.get(b"hello")?;
db.del(b"hello")?;
// Flush happens automatically - no need to call flush()!
```

### Other Languages (C FFI)

Build the shared library:
```bash
cargo build --release
```

#### String API (Recommended for Text)

```c
// C example - automatic UTF-8 handling
keylite_put_str(db, "name", "Alice");
char* value = NULL;
keylite_get_str(db, "name", &value);
printf("%s\n", value);
keylite_free_str(value);
```

```python
# Python example - automatic UTF-8 handling
keylite.keylite_put_str(db, b"name", b"Alice")
val_ptr = ctypes.c_char_p()
keylite.keylite_get_str(db, b"name", ctypes.byref(val_ptr))
print(val_ptr.value.decode('utf-8'))
keylite.keylite_free_str(val_ptr)
```

#### Binary API (For Raw Data)

For binary data, use the lower-level API with explicit lengths:
```c
keylite_put(db, key, key_len, val, val_len);
keylite_get(db, key, key_len, &val_out, &val_len_out);
```

See [FFI.md](FFI.md) for complete documentation on using Keylite from:
- C
- Python
- Node.js
- Go
- Ruby
- Java
- And more!

Examples are provided in the `examples/` directory.
