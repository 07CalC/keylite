# Implementation Plan: Document Database Layer Completion

## Overview
Enhance the document database layer (`/db`) with full CRUD operations, leveraging the existing KV transaction system for atomicity. Focus on practical, production-ready features.

---

## 🎯 Priority Order & Implementation Phases

### **Phase 1: Foundation (Critical)**
Essential components that everything else depends on.

#### ✅ 1.1 Document DB Error Types
**File:** `db/src/error.rs`

Create document-layer specific errors:
```rust
pub enum DocError {
    // Not found errors
    CollectionNotFound(String),
    DocumentNotFound(String),
    IndexNotFound(String),
    
    // Constraint violations
    UniqueConstraintViolation { field: String, value: String },
    InvalidDocumentId(String),
    
    // Schema/validation
    MissingRequiredField(String),
    InvalidFieldValue { field: String, reason: String },
    
    // Versioning (for optimistic concurrency)
    VersionMismatch { expected: u64, actual: u64 },
    
    // Wrap KV errors
    StorageError(keylite_kv::error::DbError),
    
    // Serialization
    SerializationError(String),
}
```

**Why first:** Clear error semantics improve debugging and all subsequent code will use this.

---

#### ✅ 1.2 Document Versioning Support
**File:** `db/src/collection.rs` (extend)

Add version tracking to documents:
```rust
#[derive(Serialize, Deserialize)]
pub struct DocumentEnvelope {
    pub _id: String,
    pub _version: u64,      // Auto-increment on each update
    pub _created_at: i64,
    pub _updated_at: i64,
    pub data: Value,        // Actual document
}
```

**Why first:** Enables optimistic concurrency control from the start. Better to design with versioning than retrofit later.

---

### **Phase 2: Core Update Operations (High Priority)**
The main feature requested.

#### ✅ 2.1 Full Document Replacement
**File:** `db/src/db.rs` (add method)

```rust
pub fn replace_doc(
    &self,
    collection: &str,
    id: &str,
    new_doc: Value,
    expected_version: Option<u64>,  // For optimistic locking
) -> Result<DocumentEnvelope>
```

**Implementation approach:**
1. Begin KV transaction
2. Get existing document
3. Check version if `expected_version` provided
4. Extract old indexed fields
5. Delete old index entries
6. Increment version, update timestamps
7. Write new document
8. Write new index entries
9. Commit transaction

**Key challenge:** Index updates must be atomic with document update.

---

#### ✅ 2.2 Partial Field Updates
**File:** `db/src/db.rs` (add method)

```rust
pub fn update_fields(
    &self,
    collection: &str,
    id: &str,
    updates: HashMap<String, Value>,
    expected_version: Option<u64>,
) -> Result<DocumentEnvelope>
```

**Implementation approach:**
1. Begin transaction
2. Get existing document
3. Check version
4. Merge updates into existing document (handle nested fields with dot notation like `"user.email"`)
5. Detect which indexed fields changed
6. Update only changed indexes
7. Commit

**Key features:**
- Support dot notation: `{"user.email": "new@example.com"}`
- Support special operators: `{"$unset": ["field"]}`
- Support array operations: `{"$push": {"tags": "new-tag"}}`

---

#### ✅ 2.3 Atomic Numeric Operations
**File:** `db/src/db.rs` (add method)

```rust
pub fn increment(
    &self,
    collection: &str,
    id: &str,
    field: &str,
    delta: i64,
) -> Result<i64>  // Returns new value

pub fn increment_float(
    &self,
    collection: &str,
    id: &str,
    field: &str,
    delta: f64,
) -> Result<f64>
```

**Use cases:** Counters, likes, view counts, inventory quantities.

**Implementation:** Use transaction to read-modify-write atomically.

---

### **Phase 3: Query System (Medium Priority)**
Current querying is limited. This makes it powerful.

#### ✅ 3.1 Query Builder
**File:** `db/src/query.rs` (new)

```rust
pub struct Query {
    collection: String,
    filters: Vec<Filter>,
    sort: Option<SortSpec>,
    limit: Option<usize>,
    skip: Option<usize>,
}

pub enum Filter {
    Eq { field: String, value: Value },
    Gt { field: String, value: Value },
    Lt { field: String, value: Value },
    In { field: String, values: Vec<Value> },
    Exists { field: String },
    // ... more operators
}

impl Query {
    pub fn new(collection: &str) -> Self;
    pub fn filter(mut self, filter: Filter) -> Self;
    pub fn sort(mut self, field: &str, ascending: bool) -> Self;
    pub fn limit(mut self, n: usize) -> Self;
    pub fn skip(mut self, n: usize) -> Self;
    pub fn execute(&self, db: &KeyLite) -> Result<Vec<Value>>;
}
```

**Usage example:**
```rust
let results = Query::new("users")
    .filter(Filter::Eq { field: "active".into(), value: json!(true) })
    .filter(Filter::Gt { field: "age".into(), value: json!(18) })
    .sort("created_at", false)
    .limit(10)
    .execute(&db)?;
```

**Implementation strategy:**
- Check if query can use an index (single indexed field with Eq filter)
- Otherwise fall back to collection scan with in-memory filtering
- Apply sort/limit/skip in memory (Phase 4 can optimize this)

---

#### ✅ 3.2 Aggregation Operations
**File:** `db/src/query.rs` (extend)

```rust
pub enum Aggregation {
    Count,
    Sum(String),      // field name
    Avg(String),
    Min(String),
    Max(String),
}

impl Query {
    pub fn aggregate(&self, agg: Aggregation, db: &KeyLite) -> Result<Value>;
}
```

---

### **Phase 4: Bulk Operations (Medium Priority)**
Performance optimization for batch workloads.

#### ✅ 4.1 Bulk Insert
**File:** `db/src/db.rs` (add method)

```rust
pub fn insert_many(
    &self,
    collection: &str,
    docs: Vec<Value>,
) -> Result<Vec<String>>  // Returns generated IDs
```

**Benefits:**
- Single transaction for all inserts
- Amortized index update cost
- 10-100x faster than individual inserts for large batches

---

#### ✅ 4.2 Bulk Update
**File:** `db/src/db.rs` (add method)

```rust
pub fn update_many(
    &self,
    collection: &str,
    filter: Filter,
    updates: HashMap<String, Value>,
) -> Result<usize>  // Returns count of updated docs
```

**Use case:** Update all documents matching a condition.

---

### **Phase 5: Schema Validation (Optional but Recommended)**
Prevents bad data from entering the database.

#### ✅ 5.1 JSON Schema Support
**File:** `db/src/schema.rs` (new)

```rust
pub struct Schema {
    pub required_fields: Vec<String>,
    pub field_types: HashMap<String, FieldType>,
    pub validations: HashMap<String, Vec<Validation>>,
}

pub enum FieldType {
    String, Number, Boolean, Object, Array, Null,
}

pub enum Validation {
    MinLength(usize),
    MaxLength(usize),
    Pattern(Regex),
    Min(f64),
    Max(f64),
    Enum(Vec<Value>),
}
```

**Integration:** Add optional `schema` to `CollectionMeta`, validate on insert/update.

---

### **Phase 6: Transaction Wrapper (Low Priority)**
Expose multi-document transactions at the document layer.

#### ✅ 6.1 Document Transaction API
**File:** `db/src/transaction.rs` (new)

```rust
pub struct DocTransaction<'a> {
    kv_txn: keylite_kv::transaction::Transaction<'a>,
    db: &'a KeyLite,
}

impl<'a> DocTransaction<'a> {
    pub fn insert(&mut self, collection: &str, doc: Value) -> Result<String>;
    pub fn update(&mut self, collection: &str, id: &str, ...) -> Result<()>;
    pub fn delete(&mut self, collection: &str, id: &str) -> Result<()>;
    pub fn get(&self, collection: &str, id: &str) -> Result<Option<Value>>;
    pub fn commit(self) -> Result<()>;
    pub fn abort(self);
}
```

**Use case:** Multi-document ACID operations (e.g., transfer between accounts).

---

## 📁 Final Folder Structure

```
db/
├── src/
│   ├── collection.rs      # Collection metadata, key helpers, DocumentEnvelope
│   ├── db.rs              # Main KeyLite struct with all CRUD methods
│   ├── index.rs           # Index key helpers
│   ├── error.rs           # DocError enum ⭐ NEW
│   ├── query.rs           # Query builder & aggregation ⭐ NEW
│   ├── transaction.rs     # Document transaction wrapper ⭐ NEW
│   ├── schema.rs          # Schema validation (optional) ⭐ NEW
│   └── lib.rs             # Re-exports
├── tests/
│   ├── crud_test.rs       # ⭐ NEW
│   ├── index_test.rs      # ⭐ NEW
│   ├── query_test.rs      # ⭐ NEW
│   ├── transaction_test.rs # ⭐ NEW
│   └── concurrency_test.rs # ⭐ NEW (test optimistic locking)
├── examples/
│   └── basic_usage.rs     # ⭐ NEW (documentation example)
└── Cargo.toml
```

**Additional dependencies to add:**
```toml
[dependencies]
thiserror = "2.0"          # For error types
regex = "1.10"             # For schema pattern validation (optional)
```

---

## 🔍 What's Still Missing in KV Layer (Future Work)

Analysis of KV engine shows these items missing but **NOT** blocking document layer work:

### Critical for Production (but not urgent):
1. **Compression** - Add Snappy/LZ4 to SSTable blocks (20-30% space savings)
2. **WAL replay edge cases** - Recovery looks good but needs crash-test verification
3. **Compaction trigger on read** - Currently only on SSTable count, but should also trigger on total size
4. **Iterator improvements** - `scan()` is basic, could optimize for prefix scans

### Nice-to-have:
1. **Snapshot isolation levels** - Currently single isolation level
2. **Range tombstones** - Optimize bulk deletes
3. **Block cache statistics** - For monitoring/tuning
4. **Background error handling** - Compaction/flush failures should surface to user

**Recommendation:** Focus on document layer first. KV engine is solid enough for now. Address KV improvements in a future phase.

---

## 🧪 Testing Strategy

For each phase, create comprehensive tests:

1. **Unit tests** - Each method tested in isolation
2. **Integration tests** - Full workflows (insert → update → query → delete)
3. **Concurrency tests** - Multiple threads updating same document (verify optimistic locking)
4. **Stress tests** - Large datasets (1M+ documents)
5. **Correctness tests** - Index consistency after updates

---

## 🚀 Implementation Timeline Estimate

Assuming focused work:
- **Phase 1:** 4-6 hours (foundation is crucial, don't rush)
- **Phase 2:** 6-8 hours (core update logic, most complex)
- **Phase 3:** 8-10 hours (query system is large scope)
- **Phase 4:** 3-4 hours (relatively straightforward)
- **Phase 5:** 4-5 hours (if schema validation desired)
- **Phase 6:** 2-3 hours (thin wrapper)

**Total:** ~30-35 hours for full implementation with tests.

