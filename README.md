# Document Database in Rust 

Goal: Build a document database from scratch similar to Azure Cosmos DB.

### Design doc

Rough Implementation Plan:
- [x] In memory store (just use basic hashmap)
- [ ] LSM storage engine
  - [x] Naive file persistence by putting the whole hashmap into a file and deserializing on startup
  - [x] Append only log
  - [x] WAL (?)
  - [ ] Compaction
  - [ ] Bloom filter
  - [ ] Levels for LSM tree
- [ ] Pages and heap file
- [ ] Buffer pool
- [ ] Basic query engine
- [ ] MVCC
- [ ] Query Parser
- [ ] Query Optimizer
- [ ] Cosmos DB features
