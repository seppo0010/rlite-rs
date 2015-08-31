//! # Interface to rlite
//!
//! rlite is a self-contained, serverless, zero-configuration, transactional
//! redis-compatible database engine. rlite is to Redis what SQLite is to SQL.
//! ## Example
//!
//! ```rust
//! let path = Path::new("db.rld");
//! let rlite = Rlite::file(&path).unwrap();
//!
//! rlite.write_command(&["set".as_bytes(), "key".as_bytes(), "value".as_bytes()]).unwrap();
//! assert_eq!(conn.read_reply().unwrap(), Reply::Status("OK".to_owned()));
//!
//! conn.write_command(&["get".as_bytes(), "key".as_bytes()]).unwrap();
//! assert_eq!(conn.read_reply().unwrap(), Reply::Data(b"value".to_vec()));)
//! ```
extern crate libc;

use std::mem;
use std::path::Path;
use std::ptr::copy;
use std::slice;

use libc::{c_char, c_uchar, c_int, c_ulonglong, c_void, size_t};

const RLITE_REPLY_STRING:c_int = 1;
const RLITE_REPLY_ARRAY:c_int = 2;
const RLITE_REPLY_INTEGER:c_int = 3;
const RLITE_REPLY_NIL:c_int = 4;
const RLITE_REPLY_STATUS:c_int = 5;
const RLITE_REPLY_ERROR:c_int = 6;

#[repr(C)]
struct RliteReply {
    rtype: c_int,
    integer: c_ulonglong,
    len: c_int,
    st: *const c_uchar,
    elements: size_t,
    element: *const *const RliteReply,
}

/// A command reply
#[derive(Clone, Debug, PartialEq)]
pub enum Reply {
    Nil,
    Integer(i64),
    Data(Vec<u8>),
    Status(String),
    Array(Vec<Reply>),
}

macro_rules! str_to_vec {
    ($str: expr) => {{
        let len = (*$str).len as usize;
        let mut v:Vec<u8> = Vec::with_capacity(len);
        v.set_len(len);
        copy((*$str).st, v.as_mut_ptr() as *mut u8, len);
        v
    }}
}

impl Reply {
    fn new(reply: *const RliteReply) -> Result<Self, String> {
        unsafe {
            Ok(match (*reply).rtype {
                RLITE_REPLY_STRING => {
                    Reply::Data(str_to_vec!(reply))
                },
                RLITE_REPLY_STATUS => {
                    Reply::Status(String::from_utf8(str_to_vec!(reply)).unwrap())
                },
                RLITE_REPLY_ERROR => {
                    let s = String::from_utf8(str_to_vec!(reply)).unwrap();
                    return Err(s);
                },
                RLITE_REPLY_NIL => Reply::Nil,
                RLITE_REPLY_INTEGER => Reply::Integer((*reply).integer as i64),
                RLITE_REPLY_ARRAY => {
                    let elements:usize = (*reply).elements as usize;
                    let mut v = Vec::with_capacity(elements);
                    let slice = slice::from_raw_parts((*reply).element, elements);
                    for i in 0..elements {
                        v.push(try!(Reply::new(slice[i])));
                    }
                    Reply::Array(v)
                },
                _ => return Err(format!("Unknown reply type {}", (*reply).rtype)),
            })
        }
    }
}

#[link(name = "hirlite")]
extern {
    fn rliteConnect(path: *const c_char, port: c_int) -> *mut c_void;
    fn rliteAppendCommandArgv(client: *mut c_void, argc: c_int, argv: *const *const u8, argvlen: *const size_t) -> c_int;
    fn rliteGetReply(context: *const c_void, reply: *mut *const RliteReply) -> c_int;
    fn rliteFreeReplyObject(reply: *const RliteReply);
    fn rliteFree(context: *const c_void);
}

/// A database connection
pub struct Rlite {
    rlite: *mut c_void,
}

impl Rlite {
    /// Create a new database in memory
    pub fn memory() -> Self {
        let rlite = unsafe { rliteConnect(":memory:".as_ptr() as *const c_char, 0) };
        Rlite { rlite: rlite }
    }

    /// Opens or creates a database in `path`.
    pub fn file(path: &Path) -> Result<Self, ()> {
        let f = match path.to_str() {
            Some(p) => p,
            None => return Err(()),
        };
        let rlite = unsafe { rliteConnect(::std::ffi::CString::new(f).unwrap().as_ptr() as *const c_char, 0) };
        if rlite != 0 as *mut _ {
            Ok(Rlite { rlite: rlite })
        } else {
            Err(())
        }
    }

    /// Executes the command. It returns either success or error, with no detail.
    /// If it succeeded, use `read_reply` to get the response (if any).
    pub fn write_command(&self, command: &[&[u8]]) -> Result<(), ()> {
        let mut argv = Vec::new();
        let mut argvlen = Vec::new();
        for c in command {
            argv.push(c.as_ptr());
            argvlen.push(c.len() as size_t);
        }
        unsafe {
            if rliteAppendCommandArgv(self.rlite, command.len() as c_int, argv.as_ptr(), argvlen.as_ptr()) == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    /// Reads the response of the first unread command.
    /// Each command response is appended to a queue. This command pops from
    /// that queue.
    pub fn read_reply(&self) -> Result<Reply, String> {
        unsafe {
            let mut reply = mem::zeroed();
            if rliteGetReply(self.rlite, &mut reply) == 0 {
                if reply == 0 as *mut _ {
                    return Ok(Reply::Nil);
                }
                let r = Reply::new(reply);
                rliteFreeReplyObject(reply);
                r
            } else {
                Err("Failed".to_owned())
            }
        }
    }
}

impl Drop for Rlite {
    fn drop(&mut self) {
        unsafe { rliteFree(self.rlite); }
    }
}
