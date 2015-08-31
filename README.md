# Interface to rlite

rlite is a self-contained, serverless, zero-configuration, transactional
redis-compatible database engine. rlite is to Redis what SQLite is to SQL.

## Getting Started

rlite-rs is available on
[crates.io](https://crates.io/crates/rlite).
Add the following dependency to your Cargo manifest:

```toml
[dependencies]
rlite = "0.0.1"
```

## Example

```rust
let path = Path::new("db.rld");
let rlite = Rlite::file(&path).unwrap();

rlite.write_command(&["set".as_bytes(), "key".as_bytes(), "value".as_bytes()]).unwrap();
assert_eq!(conn.read_reply().unwrap(), Reply::Status("OK".to_owned()));

conn.write_command(&["get".as_bytes(), "key".as_bytes()]).unwrap();
assert_eq!(conn.read_reply().unwrap(), Reply::Data(b"value".to_vec()));
```
