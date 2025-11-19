const isBun = typeof Bun !== "undefined";

if (isBun) {
  // Bun has FFI issues with this library, recommend using Node.js
  console.error("⚠️  Warning: Bun FFI has known issues with keylite-kv.");
  console.error("    Please compile with Bun and run with Node.js:");
  console.error("    bun build your-file.ts --outfile output.js --target=node && node output.js");
}

export { Keylite, KeyliteIterator } from "./keylite";
export { default as keyliteFFI } from "./keylite-ffi";
