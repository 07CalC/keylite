# Keylite JavaScript Bindings

Node.js bindings for Keylite, a high-performance LSM-tree based key-value store.

## 🚀 Quick Start

```bash
cd bindings/js
./quickstart.sh
```

Or manually:

```bash
npm install          # Install dependencies
npm run example      # Run the example
npm test            # Run tests
```

## ⚠️ Important: Bun Compatibility

These bindings use `ffi-napi` which **works perfectly with Node.js** but has issues when run directly with Bun due to bugs in Bun's FFI implementation (segfaults/double-free errors related to background thread cleanup).

**✅ Recommended approach:**
1. Use Bun for development/building (it's faster)
2. Run the compiled code with Node.js (it's stable)

The npm scripts handle this automatically!

```bash
npm run example   # Builds with Bun, runs with Node
npm test         # Builds with Bun, runs with Node
```

**Manual workflow:**
```bash
# Build TypeScript with Bun
bun build myapp.ts --outfile myapp.js --target=node

# Run with Node.js
node myapp.js
```

See [RUNNING_WITH_NODE.md](./RUNNING_WITH_NODE.md) for detailed instructions.

## Usage

### Basic Operations

```typescript
import { Keylite } from "keylite-kv";

// Open database
const db = new Keylite();
db.open("./mydb");

// String operations
db.putStr("hello", "world");
const value = db.getStr("hello");
console.log(value); // "world"

db.delStr("hello");

// Binary operations
const key = Buffer.from("mykey");
const val = Buffer.from("myvalue");
db.put(key, val);

const result = db.get(key);
console.log(result?.toString()); // "myvalue"

db.del(key);

// Close database
db.close();
```

### Scanning / Iteration

```typescript
// Scan all keys
for (const { key, value } of db.scan()) {
  console.log(key.toString(), value.toString());
}

// Scan with range (from "a" to "z")
for (const { key, value } of db.scanStr("a", "z")) {
  console.log(key.toString(), value.toString());
}

// Manual iteration
const iter = db.scan();
let item;
while ((item = iter.next()) !== null) {
  console.log(item.key.toString(), item.value.toString());
}
iter.close();
```

## API Reference

### `Keylite`

#### `open(path: string): void`
Opens a database at the specified path.

#### `close(): void`
Closes the database connection.

#### Binary Operations

- `put(key: Buffer, value: Buffer): void` - Store a key-value pair
- `get(key: Buffer): Buffer | null` - Retrieve a value by key
- `del(key: Buffer): void` - Delete a key-value pair

#### String Operations

- `putStr(key: string, value: string): void` - Store a string key-value pair
- `getStr(key: string): string | null` - Retrieve a string value by key
- `delStr(key: string): void` - Delete a string key-value pair

#### Scanning

- `scan(start?: Buffer, end?: Buffer): KeyliteIterator` - Create an iterator over a range
- `scanStr(start?: string, end?: string): KeyliteIterator` - Create an iterator over a string range

### `KeyliteIterator`

#### `next(): { key: Buffer; value: Buffer } | null`
Get the next key-value pair, or null if iteration is complete.

#### `close(): void`
Close the iterator and free resources.

#### Iterator Protocol
`KeyliteIterator` implements the JavaScript iterator protocol, so you can use it with `for...of` loops.

## Running the Example

```bash
npm run example
```

This will compile the TypeScript example and run it with Node.js.

## Running Tests

```bash
npm test
```

This will run a comprehensive test suite covering all functionality.

## Development

To use these bindings in your own project:

1. Install dependencies: `npm install`
2. Import the library: `import { Keylite } from "keylite-kv"`
3. Build your TypeScript file for Node.js target
4. Run with Node.js

This project was created using `bun init` but should be run with Node.js due to current Bun FFI limitations.
