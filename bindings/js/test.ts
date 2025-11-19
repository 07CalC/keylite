import { Keylite } from "./index";
import { strict as assert } from "assert";

console.log("Running Keylite tests...\n");

// Test 1: Open and Close
console.log("Test 1: Open and Close");
const db = new Keylite();
db.open("./test_db");
assert.ok(true, "Database opened successfully");
db.close();
assert.ok(true, "Database closed successfully");
console.log("✓ Pass\n");

// Test 2: String Operations
console.log("Test 2: String Operations");
db.open("./test_db");
db.putStr("key1", "value1");
const val1 = db.getStr("key1");
assert.strictEqual(val1, "value1", "Get should return the correct value");

const val2 = db.getStr("nonexistent");
assert.strictEqual(val2, null, "Get should return null for non-existent keys");

db.delStr("key1");
const val3 = db.getStr("key1");
assert.strictEqual(val3, null, "Get should return null after delete");
db.close();
console.log("✓ Pass\n");

// Test 3: Binary Operations
console.log("Test 3: Binary Operations");
db.open("./test_db");
const key = Buffer.from("binarykey");
const value = Buffer.from([0x01, 0x02, 0x03, 0x04, 0x05]);
db.put(key, value);

const result = db.get(key);
assert.ok(result !== null, "Get should return a buffer");
assert.deepStrictEqual(result, value, "Retrieved value should match");

db.del(key);
const result2 = db.get(key);
assert.strictEqual(result2, null, "Get should return null after delete");
db.close();
console.log("✓ Pass\n");

// Test 4: Multiple Keys
console.log("Test 4: Multiple Keys");
db.open("./test_db");
db.putStr("user:1", "Alice");
db.putStr("user:2", "Bob");
db.putStr("user:3", "Charlie");
db.putStr("post:1", "First post");
db.putStr("post:2", "Second post");

assert.strictEqual(db.getStr("user:1"), "Alice");
assert.strictEqual(db.getStr("user:2"), "Bob");
assert.strictEqual(db.getStr("user:3"), "Charlie");
assert.strictEqual(db.getStr("post:1"), "First post");
assert.strictEqual(db.getStr("post:2"), "Second post");
console.log("✓ Pass\n");

// Test 5: Scan All
console.log("Test 5: Scan All");
const allKeys = [];
for (const { key, value } of db.scan()) {
  allKeys.push(key.toString());
}
assert.ok(allKeys.length >= 5, "Should have at least 5 keys");
assert.ok(allKeys.includes("user:1"), "Should include user:1");
assert.ok(allKeys.includes("post:2"), "Should include post:2");
console.log("✓ Pass\n");

// Test 6: Scan Range
console.log("Test 6: Scan Range");
const userKeys = [];
for (const { key, value } of db.scanStr("user:", "user:z")) {
  userKeys.push(key.toString());
}
assert.strictEqual(userKeys.length, 3, "Should have 3 user keys");
assert.ok(userKeys.every(k => k.startsWith("user:")), "All keys should start with user:");
console.log("✓ Pass\n");

// Test 7: Manual Iterator
console.log("Test 7: Manual Iterator");
const iter = db.scanStr("post:", "post:z");
let count = 0;
let item;
while ((item = iter.next()) !== null) {
  count++;
  assert.ok(item.key.toString().startsWith("post:"), "Key should start with post:");
}
iter.close();
assert.strictEqual(count, 2, "Should have 2 post keys");
console.log("✓ Pass\n");

// Test 8: Overwrite Value
console.log("Test 8: Overwrite Value");
db.putStr("overwrite", "initial");
assert.strictEqual(db.getStr("overwrite"), "initial");
db.putStr("overwrite", "updated");
assert.strictEqual(db.getStr("overwrite"), "updated");
console.log("✓ Pass\n");

// Test 9: Special Characters
console.log("Test 9: Special Characters");
db.putStr("special", "Hello 世界 🌍");
assert.strictEqual(db.getStr("special"), "Hello 世界 🌍");
console.log("✓ Pass\n");

// Test 10: Persistence
console.log("Test 10: Persistence");
db.putStr("persistent", "data");
db.close();

// Reopen and check
db.open("./test_db");
assert.strictEqual(db.getStr("persistent"), "data", "Data should persist after close/open");
db.close();
console.log("✓ Pass\n");

console.log("All tests passed! ✨");
