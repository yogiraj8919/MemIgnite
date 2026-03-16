# 🧠 MemIgnite

**Redis-Inspired In-Memory Key-Value Database (Rust)**

**Author:** Saiyogiraj
**Language:** Rust
**Runtime:** Tokio

---

# Overview

**MemIgnite** is a Redis-inspired in-memory key-value database written in Rust.
The project was built **for educational and learning purposes** to understand how modern databases are designed internally.

The goal of this project is to explore important **systems and database engineering concepts** such as:

* Async networking
* Concurrent data structures
* Key expiration strategies
* Database persistence
* Crash recovery
* Log compaction
* Storage engine architecture

MemIgnite is not intended to replace Redis or production databases.
Instead, it serves as a **learning project to study database internals and systems programming in Rust.**

---

# Key Learning Objectives

This project was built to gain hands-on experience with:

• Building an **async TCP server** using Tokio
• Designing a **concurrent in-memory storage engine**
• Implementing **TTL expiration mechanisms**
• Implementing **Append-Only File (AOF) persistence**
• Understanding **crash recovery using log replay**
• Implementing **log compaction (AOF Rewrite)**
• Designing modular database architecture

---

# System Architecture

MemIgnite follows a layered architecture similar to real databases.

```id="arch1"
Client
   │
   ▼
TCP Server (Tokio)
   │
   ▼
Command Parser
   │
   ▼
Command Handler
   │
   ▼
Storage Engine
   ├── DashMap (Concurrent Storage)
   ├── TTL Expiration Scheduler
   ├── Background Expiration Worker
   └── Persistence Layer
           │
           ├── Append Only File (AOF)
           └── AOF Rewrite (Log Compaction)
```

---

# Features Implemented

## Async Networking

MemIgnite uses the **Tokio runtime** to support multiple concurrent client connections.

Each connection is handled asynchronously.

---

## Concurrent Storage Engine

The storage engine is built using **DashMap**, allowing concurrent reads and writes without global locking.

```id="store1"
DashMap<String, Entry>
```

Each entry stores:

```id="entry1"
value
optional expiration timestamp
```

---

# Supported Commands

| Command                  | Description              |
| ------------------------ | ------------------------ |
| SET key value            | Store key-value pair     |
| SET key value EX seconds | Store with expiration    |
| GET key                  | Retrieve a value         |
| DEL key                  | Delete a key             |
| LPUSH key value          | Push value to list       |
| RDROP key                | Remove rightmost element |
| PING                     | Health check             |
| ECHO message             | Echo message             |
| REWRITEAOF               | Compact persistence log  |
| HELP                     | Display help             |
| QUIT                     | Close connection         |

---

# TTL Expiration

Keys can expire automatically using the `EX` option.

Example:

```id="ttl1"
SET session_token abc EX 60
```

Two expiration strategies are implemented:

### Lazy Expiration

Keys are checked for expiration during read operations.

### Background Expiration Worker

A periodic task removes expired keys.

---

# Persistence: Append Only File (AOF)

MemIgnite uses an **append-only persistence log**.

Every write operation is appended to:

```id="aof1"
appendonly.aof
```

Example log:

```id="aof2"
SET user raj
SET age 24
DEL age
LPUSH users alice
```

This ensures durability and allows crash recovery.

---

# Crash Recovery

When the server restarts, MemIgnite **replays the AOF log** to rebuild the database state.

Example replay:

```id="replay1"
SET a 1
SET a 2
SET a 3
```

Final state:

```id="replay2"
a = 3
```

---

# AOF Rewrite (Log Compaction)

Over time the AOF file grows due to repeated operations.

MemIgnite supports **log compaction** using the `REWRITEAOF` command.

Before rewrite:

```id="rewrite1"
SET a 1
SET a 2
SET a 3
SET a 4
```

After rewrite:

```id="rewrite2"
SET a 4
```

Benefits:

• Reduced disk usage
• Faster database startup
• Removal of redundant operations

---

# Example Session

```id="cli1"
SET name raj
OK

GET name
raj

LPUSH users alice
1

LPUSH users bob
2

RDROP users
alice

SET token abc EX 10
OK

REWRITEAOF
AOF rewrite completed
```

---

# Project Structure

```id="structure1"
src/

main.rs        → Entry point
server.rs      → TCP server
handler.rs     → Command handling
parser.rs      → Command parser
command.rs     → Command definitions
store.rs       → Storage engine
aof.rs         → Persistence and log rewrite
```

---

# Running the Project

### Clone repository

```id="run1"
git clone <repo-url>
cd memignite
```

### Start server

```id="run2"
cargo run
```

The MemIgnite CLI will start and accept commands.

---

# Educational Purpose

MemIgnite is a **learning project designed to understand database internals and systems programming concepts using Rust**.

It demonstrates how core database features such as persistence, concurrency, expiration, and log compaction can be implemented from scratch.

The project focuses on **understanding system design and database architecture rather than production use.**

---

# Future Improvements

Planned features include:

• INFO command for server metrics
• RESP protocol support
• Redis CLI compatibility
• Performance benchmarking
• Snapshot persistence
• Replication support
• Memory usage metrics

---

# License

This project is intended for **educational and learning purposes only**.
