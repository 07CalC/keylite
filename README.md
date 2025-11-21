# keylite

A lightweight, embedded key-value (document db wip) database written in Rust with support for multiple programming languages.

## Architecture

Keylite uses a modern SSTable (Sorted String Table) architecture:

- **Memtable**: Recent writes stored in memory (BTreeMap) up to 4MB
- **SSTables**: Immutable sorted files on disk with:
  - 16KB data blocks with sorted key-value pairs
  - Sparse index for fast lookups (one entry per block)
  - Bloom filters for efficient negative lookups
  - CRC32 checksums for data integrity
- **Automatic flushing**: Memtable flushes to SSTable when size threshold reached (via separate thread)
- **Automatic compaction**: Merges SSTables when count exceeds threshold (via separate thread)

## Crates:

- keylite-kv: Core key-value storage engine with SSTable implementation `/kv`

- keylite: CLI tool for interactive with the keylite-kv (document db will also be added here) database `/cli`

- keylite-db (wip): Document database layer built on top of keylite-kv.

## SSTable file format

```
+--------------------------+
| Data Block 0             |
|  key_len | val_len | K|V |
+--------------------------+
| Data Block 1             |
+--------------------------+
| ...                      |
+--------------------------+
| Bloom Filter             |
+--------------------------+
| Block Index              |
+--------------------------+
| Footer (pointers)        |
+--------------------------+
```


## Roadmap

- [x] SSTable reader, writer
- [x] bloom filters
- [x] compaction
- [ ] transactions
- [ ] compression
- [ ] Document db layer `/db` (wip)
- [ ] bindings for other languages

## Benchmarks

> for a database with 2 million entries

- DB open time: `501us`
- write throughput: `174887` ops/sec
- read throughput: `103028` ops/sec
- del throughput: `99972` ops/sec

```
--- Write Latency (ns) ---
p50: 4927 ns,  p90: 5895 ns,  p99: 14512 ns

--- Read Latency (ns) ---
p50: 6897 ns,  p90: 18503 ns,  p99: 325617 ns

--- Delete Latency (ns) ---
p50: 4849 ns,  p90: 5788 ns,  p99: 12901 ns
```


