import lib from "./keylite-ffi";
import ref from "ref-napi"

type Handle = Buffer;

enum KeyliteResult {
  Ok = 0,
  ErrNull = 1,
  ErrIo = 2,
  ErrUtf8 = 3,
  ErrOther = 4,
}

function checkResult(result: number, operation: string): void {
  if (result !== KeyliteResult.Ok) {
    const errorMessages = {
      [KeyliteResult.ErrNull]: "Null pointer error",
      [KeyliteResult.ErrIo]: "I/O error",
      [KeyliteResult.ErrUtf8]: "UTF-8 encoding error",
      [KeyliteResult.ErrOther]: "Unknown error",
    };
    throw new Error(`${operation} failed: ${errorMessages[result] || "Unknown error"}`);
  }
}

export class KeyliteIterator {
  private handle: Handle;
  private db: Keylite;

  constructor(handle: Handle, db: Keylite) {
    this.handle = handle;
    this.db = db;
  }

  next(): { key: Buffer; value: Buffer } | null {
    const keyOut = ref.alloc(ref.refType(ref.types.uchar));
    const keyLenOut = ref.alloc(ref.types.size_t);
    const valOut = ref.alloc(ref.refType(ref.types.uchar));
    const valLenOut = ref.alloc(ref.types.size_t);

    const result = lib.keylite_iter_next(
      this.handle,
      keyOut,
      keyLenOut,
      valOut,
      valLenOut
    );

    checkResult(result, "Iterator next");

    const keyPtr = keyOut.deref();
    const keyLen = keyLenOut.deref();

    // If keyPtr is null, we've reached the end
    if (ref.isNull(keyPtr) || keyLen === 0) {
      return null;
    }

    const valPtr = valOut.deref();
    const valLen = valLenOut.deref();

    // Copy the data to JavaScript buffers
    const key = Buffer.from(ref.reinterpret(keyPtr, keyLen, 0));
    const value = Buffer.from(ref.reinterpret(valPtr, valLen, 0));

    // Free the C-allocated memory
    lib.keylite_free_value(keyPtr, keyLen);
    lib.keylite_free_value(valPtr, valLen);

    return { key, value };
  }

  close(): void {
    if (this.handle) {
      lib.keylite_iter_free(this.handle);
    }
  }

  // Iterator protocol for for...of loops
  *[Symbol.iterator](): Generator<{ key: Buffer; value: Buffer }> {
    let item: { key: Buffer; value: Buffer } | null;
    while ((item = this.next()) !== null) {
      yield item;
    }
    this.close();
  }
}

export class Keylite {
  private handle: Handle | null = null;

  open(path: string): void {
    if (this.handle) {
      throw new Error("Database is already open");
    }

    const dbOut = ref.alloc(ref.refType(ref.types.void));
    const result = lib.keylite_open(path, dbOut);

    checkResult(result, "Open database");

    this.handle = dbOut.deref();
    if (ref.isNull(this.handle)) {
      throw new Error("Failed to open database");
    }
  }

  close(): void {
    if (!this.handle) return;
    lib.keylite_close(this.handle);
    this.handle = null;
  }

  private ensureOpen(): void {
    if (!this.handle) {
      throw new Error("Database is not open");
    }
  }

  // Binary key-value operations
  put(key: Buffer, value: Buffer): void {
    this.ensureOpen();
    const result = lib.keylite_put(
      this.handle,
      key,
      key.length,
      value,
      value.length
    );
    checkResult(result, "Put");
  }

  get(key: Buffer): Buffer | null {
    this.ensureOpen();

    const valOut = ref.alloc(ref.refType(ref.types.uchar));
    const valLenOut = ref.alloc(ref.types.size_t);

    const result = lib.keylite_get(
      this.handle,
      key,
      key.length,
      valOut,
      valLenOut
    );

    checkResult(result, "Get");

    const valPtr = valOut.deref();
    const valLen = valLenOut.deref();

    // If valPtr is null, key doesn't exist
    if (ref.isNull(valPtr) || valLen === 0) {
      return null;
    }

    // Copy the data to a JavaScript buffer
    const value = Buffer.from(ref.reinterpret(valPtr, valLen, 0));

    // Free the C-allocated memory
    lib.keylite_free_value(valPtr, valLen);

    return value;
  }

  del(key: Buffer): void {
    this.ensureOpen();
    const result = lib.keylite_del(this.handle, key, key.length);
    checkResult(result, "Delete");
  }

  // String key-value operations
  putStr(key: string, value: string): void {
    this.ensureOpen();
    const result = lib.keylite_put_str(this.handle, key, value);
    checkResult(result, "Put string");
  }

  getStr(key: string): string | null {
    this.ensureOpen();

    const valOut = ref.alloc(ref.refType(ref.types.char));
    const result = lib.keylite_get_str(this.handle, key, valOut);

    checkResult(result, "Get string");

    const valPtr = valOut.deref();

    // If valPtr is null, key doesn't exist
    if (ref.isNull(valPtr)) {
      return null;
    }

    // Read the string - readCString reads until null terminator
    const value = ref.readCString(valPtr, 0);

    // Free the C-allocated memory
    lib.keylite_free_str(valPtr);

    return value;
  }

  delStr(key: string): void {
    this.ensureOpen();
    const result = lib.keylite_del_str(this.handle, key);
    checkResult(result, "Delete string");
  }

  // Scan operations
  scan(start?: Buffer, end?: Buffer): KeyliteIterator {
    this.ensureOpen();

    const iterOut = ref.alloc(ref.refType(ref.types.void));

    const startPtr = start ? start : null;
    const startLen = start ? start.length : 0;
    const endPtr = end ? end : null;
    const endLen = end ? end.length : 0;

    const result = lib.keylite_scan(
      this.handle,
      startPtr,
      startLen,
      endPtr,
      endLen,
      iterOut
    );

    checkResult(result, "Scan");

    const iterHandle = iterOut.deref();
    return new KeyliteIterator(iterHandle, this);
  }

  scanStr(start?: string, end?: string): KeyliteIterator {
    this.ensureOpen();

    const iterOut = ref.alloc(ref.refType(ref.types.void));

    const result = lib.keylite_scan_str(
      this.handle,
      start || null,
      end || null,
      iterOut
    );

    checkResult(result, "Scan string");

    const iterHandle = iterOut.deref();
    return new KeyliteIterator(iterHandle, this);
  }
}
