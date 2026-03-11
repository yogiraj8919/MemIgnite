# 🧠 MemIgnite

A high-performance **in-memory key-value database engine written in Rust**.

MemIgnite explores the internal architecture of modern data systems including **concurrent storage, TTL expiration scheduling, asynchronous networking, and write-ahead logging (WAL)**.

The project is inspired by systems such as **Redis**, but implemented from scratch for learning and experimentation with **Rust systems programming**.

---

# 🚀 Highlights

* Async **TCP database server** built with Tokio
* **Concurrent storage engine** using DashMap
* **BinaryHeap expiration scheduler** (no O(n) scans)
* **Lazy + active TTL expiration**
* **Write-Ahead Log (WAL/AOF)** for crash recovery
* Multiple data types (**String + List**)
* Redis-style commands (`SET`, `GET`, `DEL`, `LPUSH`, `RPOP`)

---

# ⚡ Architecture

```
Client
  │
  ▼
Async TCP Server (Tokio)
  │
  ▼
Command Parser
  │
  ▼
Storage Engine
 ├─ DashMap (Concurrent KV Store)
 ├─ TTL Scheduler (BinaryHeap)
 └─ WAL Persistence
```

---

# 📦 Supported Commands

```
PING
SET <key> <value>
SET <key> <value> EX <seconds>
GET <key>
DEL <key>
LPUSH <key> <value>
RPOP <key>
ECHO <message>
QUIT
```

---

# ⏱ Expiration Design

MemIgnite uses a **heap-based TTL scheduler** instead of scanning the entire database.

Expiration events are stored as:

```
(expires_at, key)
```

Benefits:

* earliest expiration processed first
* avoids O(n) scans
* predictable performance

Lazy expiration during `GET` guarantees correctness even if the background worker is delayed.

---

# 💾 Persistence

MemIgnite implements **Write-Ahead Logging (WAL)**.

Write path:

```
Client command
      ↓
Append operation to WAL
      ↓
Apply update to memory store
```

On restart the database **replays the WAL** to rebuild the state.

---

# 📊 Complexity

| Operation    | Complexity |
| ------------ | ---------- |
| GET          | O(1)       |
| SET          | O(1)       |
| SET with TTL | O(log n)   |
| Expiration   | O(log n)   |

---

# 📁 Project Structure

```
src/
├── main.rs        # Server entry point
├── server.rs      # Async TCP server
├── handler.rs     # Client request handler
├── parser.rs      # Command parsing
├── command.rs     # Command definitions
├── store.rs       # Storage engine
└── aof.rs         # WAL persistence
```

---

# ▶️ Running

Start the database server:

```
cargo run
```

Connect using netcat:

```
nc 127.0.0.1 6379
```

Example:

```
SET user raj
OK

GET user
raj
```

---

# 🧰 Tech Stack

| Component            | Technology            |
| -------------------- | --------------------- |
| Language             | Rust                  |
| Async Runtime        | Tokio                 |
| Concurrent Storage   | DashMap               |
| Expiration Scheduler | BinaryHeap            |
| Persistence          | WAL (Append Only Log) |
| Networking           | TCP                   |

---

# 🎯 Motivation

MemIgnite was built to explore:

* database architecture
* concurrent data structures
* async network servers
* expiration scheduling algorithms
* crash recovery with WAL

The goal is to build a **small but realistic database engine from scratch**.

---

# ⚠️ Status

MemIgnite is an **experimental learning project** and not production-ready.

---

# 👨‍💻 Author

Saiyogiraj
