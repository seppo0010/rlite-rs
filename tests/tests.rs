extern crate rlite;

use std::fs::remove_file;
use std::path::Path;

use rlite::{Rlite, Reply};

#[test]
fn persists() {
    let path = Path::new("./tmp");
    let _ = remove_file(&path);

    {
        let conn = Rlite::file(&path).unwrap();
        conn.write_command(&[
                    "set".as_bytes(), "key".as_bytes(), "value".as_bytes(),
                    ]).unwrap();
        conn.write_command(&[
                    "get".as_bytes(), "key".as_bytes(),
                    ]).unwrap();
        assert_eq!(conn.read_reply().unwrap(), Reply::Status("OK".to_owned()));
        assert_eq!(conn.read_reply().unwrap(), Reply::Data(b"value".to_vec()));
    }
    {
        let conn = Rlite::file(&path).unwrap();
        conn.write_command(&[
                    "get".as_bytes(), "key".as_bytes(),
                    ]).unwrap();
        assert_eq!(conn.read_reply().unwrap(), Reply::Data(b"value".to_vec()));
    }

    let _ = remove_file(&path);
}

#[test]
fn reply_error() {
    let conn = Rlite::memory();
    conn.write_command(&[
                "ping".as_bytes(), "1".as_bytes(), "2".as_bytes(),
                ]).unwrap();
    assert!(conn.read_reply().unwrap_err().contains("ERR wrong number of arguments"));
}

#[test]
fn reply_status() {
    let conn = Rlite::memory();
    conn.write_command(&[
                "ping".as_bytes(),
                ]).unwrap();
    assert_eq!(conn.read_reply().unwrap(), Reply::Status("PONG".to_string()));
}

#[test]
fn reply_string() {
    let conn = Rlite::memory();
    conn.write_command(&[
                "set".as_bytes(), "key".as_bytes(), "value".as_bytes(),
                ]).unwrap();
    conn.write_command(&[
                "get".as_bytes(), "key".as_bytes(),
                ]).unwrap();
    assert_eq!(conn.read_reply().unwrap(), Reply::Status("OK".to_string()));
    assert_eq!(conn.read_reply().unwrap(), Reply::Data(b"value".to_vec()));
}

#[test]
fn reply_nil() {
    let conn = Rlite::memory();
    conn.write_command(&[
                "get".as_bytes(), "key".as_bytes(),
                ]).unwrap();
    assert_eq!(conn.read_reply().unwrap(), Reply::Nil);
}

#[test]
fn reply_integer() {
    let conn = Rlite::memory();
    conn.write_command(&[
                "LPUSH".as_bytes(), "key".as_bytes(), "1".as_bytes(), "2".as_bytes(),
                ]).unwrap();
    assert_eq!(conn.read_reply().unwrap(), Reply::Integer(2));
}

#[test]
fn reply_array() {
    let conn = Rlite::memory();
    conn.write_command(&[
                "RPUSH".as_bytes(), "key".as_bytes(),
                "1".as_bytes(), "2".as_bytes(), "3".as_bytes(),
                ]).unwrap();
    conn.write_command(&[
                "LRANGE".as_bytes(), "key".as_bytes(),
                "0".as_bytes(), "-1".as_bytes(),
                ]).unwrap();
    assert_eq!(conn.read_reply().unwrap(), Reply::Integer(3));
    assert_eq!(conn.read_reply().unwrap(), Reply::Array(vec![
                Reply::Data(b"1".to_vec()),
                Reply::Data(b"2".to_vec()),
                Reply::Data(b"3".to_vec()),
                ]));
}
