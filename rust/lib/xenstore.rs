use std::ffi::CStr;
use std::ffi::CString;

use super::bindings;

// =============================================================================

pub struct Error {
  // TODO: Add impl.
}

impl Error {
  pub fn new () -> Self {
    Self { }
  }
}

impl std::fmt::Display for Error {
  fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "error")
  }
}

pub type Result<T> = std::result::Result<T, Error>;

// -----------------------------------------------------------------------------

pub struct Xenstore {
  xs: *mut bindings::xs_handle
}

impl Xenstore {
  pub fn new () -> std::result::Result<Self, &'static str> {
    unsafe {
      let xs = bindings::xs_open(0);
      if !xs.is_null() { Ok(Self { xs }) } else { Err("Failed to open xenstore") }
    }
  }

  pub fn get_domain_path (&self, dom_id: u32) -> String {
    unsafe {
      String::from(CStr::from_ptr(bindings::xs_get_domain_path(self.xs, dom_id)).to_str().unwrap())
    }
  }

  pub fn read (&self, path: &str) -> Result<String> {
    self.read_transaction(bindings::XBT_NULL, &path)
  }

  pub fn write (&self, path: &str, value: &str) -> Result<()> {
    self.write_transaction(bindings::XBT_NULL, &path, &value)
  }

  pub fn rm (&self, path: &str) -> Result<()> {
    self.rm_transaction(bindings::XBT_NULL, &path)
  }

  fn read_transaction (&self, tr: bindings::xs_transaction_t, path: &str) -> Result<String> {
    unsafe {
      let mut len: u32 = 0;
      let buf = bindings::xs_read(self.xs, tr, CString::new(path).unwrap().as_ptr(), &mut len);
      if buf.is_null() {
        Err(Error::new())
      } else {
        let size = len as usize;
        let value = match String::from_utf8(std::slice::from_raw_parts(buf as *mut u8, size).to_vec()) {
          Ok(value) => value,
          Err(e) => return Err(Error::new())
        };
        libc::free(buf);
        Ok(value)
      }
    }
  }

  fn write_transaction (&self, tr: bindings::xs_transaction_t, path: &str, value: &str) -> Result<()> {
    unsafe {
      if bindings::xs_write(self.xs, tr, CString::new(path).unwrap().as_ptr(), value.as_ptr() as *const libc::c_void, value.len() as u32) {
        Ok(())
      } else {
        Err(Error::new())
      }
    }
  }

  fn rm_transaction (&self, tr: bindings::xs_transaction_t, path: &str) -> Result<()> {
    unsafe {
      if bindings::xs_rm(self.xs, tr, CString::new(path).unwrap().as_ptr()) {
        Ok(())
      } else {
        Err(Error::new())
      }
    }
  }
}

impl Drop for Xenstore {
  fn drop (&mut self) {
    unsafe { bindings::xs_close(self.xs); }
  }
}

unsafe impl Send for Xenstore {}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum TransactionStatus {
  Pending,
  Committed,
  Aborted
}

pub struct Transaction<'a> {
  store: &'a Xenstore,
  tr: bindings::xs_transaction_t,
  status: TransactionStatus
}

impl<'a> Transaction<'a> {
  pub fn new (store: &'a Xenstore) -> Result<Self> {
    unsafe {
      match bindings::xs_transaction_start(store.xs) {
        bindings::XBT_NULL => Err(Error::new()),
        tr => Ok(Self { store, tr, status: TransactionStatus::Pending })
      }
    }
  }

  pub fn read (&self, path: &str) -> Result<String> {
    self.store.read_transaction(self.tr, path)
  }

  pub fn write (&self, path: &str, value: &str) -> Result<()> {
    self.store.write_transaction(self.tr, path, value)
  }

  pub fn rm (&self, path: &str) -> Result<()> {
    self.store.rm_transaction(self.tr, path)
  }

  pub fn commit (&self) -> Result<()> {
    unsafe {
      if bindings::xs_transaction_end(self.store.xs, self.tr, false) {
        Ok(())
      } else {
        Err(Error::new())
      }
    }
  }

  pub fn get_status (&self) -> TransactionStatus {
    self.status
  }
}

impl Drop for Transaction<'_> {
  fn drop (&mut self) {
    unsafe {
      if self.status == TransactionStatus::Pending {
        bindings::xs_transaction_end(self.store.xs, self.tr, true);
      }
    }
  }
}
