# Document Database in Rust 

Goal: Build a document database from scratch similar to Azure Cosmos DB.

### Design doc

Rough Implementation Plan:
- [ ] In memory store (just use basic hashmap)
- [ ] Naive file persistence by putting the whole hashmap into a file and deserializing on startup
- [ ] Append only log
- [ ] Pages and heap file
- [ ] B Tree
- [ ] WAL (?)
- [ ] Buffer pool
- [ ] Basic query engine
- [ ] MVCC
- [ ] Query Parser
- [ ] Query Optimizer
- [ ] Cosmos DB features

