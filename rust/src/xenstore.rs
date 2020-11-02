use std::ffi::CStr;
use std::ffi::CString;

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
  xs: *mut xenstore_sys::xs_handle
}

impl Xenstore {
  pub fn new () -> Self {
    unsafe { Self { xs: xenstore_sys::xs_open(0) } }
  }

  pub fn get_domain_path (&self, dom_id: u32) -> String {
    unsafe {
      String::from(CStr::from_ptr(xenstore_sys::xs_get_domain_path(self.xs, dom_id)).to_str().unwrap())
    }
  }
}

impl Drop for Xenstore {
  fn drop (&mut self) {
    unsafe { xenstore_sys::xs_close(self.xs); }
  }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum TransactionStatus {
  Pending,
  Committed,
  Aborted
}

pub struct Transaction<'a> {
  store: &'a Xenstore,
  tr: xenstore_sys::xs_transaction_t,
  status: TransactionStatus
}

impl<'a> Transaction<'a> {
  pub fn new (store: &'a Xenstore) -> Result<Self> {
    unsafe {
      match xenstore_sys::xs_transaction_start(store.xs) {
        xenstore_sys::XBT_NULL => Err(Error::new()),
        tr => Ok(Self { store, tr, status: TransactionStatus::Pending })
      }
    }
  }

  pub fn read (&self, path: &str) -> Result<String> {
    unsafe {
      let mut len: u32 = 0;
      let buf = xenstore_sys::xs_read(self.store.xs, self.tr, CString::new(path).unwrap().as_ptr(), &mut len);
      if buf.is_null() {
        Err(Error::new())
      } else {
        let size = len as usize;
        let value = String::from_utf8_unchecked(std::vec::Vec::from_raw_parts(buf as *mut u8, size, size));
        libc::free(buf);
        Ok(value)
      }
    }
  }

  pub fn write (&self, path: &str, value: &str) -> Result<()> {
    unsafe {
      if xenstore_sys::xs_write(self.store.xs, self.tr, CString::new(path).unwrap().as_ptr(), value.as_ptr() as *const libc::c_void, value.len() as u32) {
        Ok(())
      } else {
        Err(Error::new())
      }
    }
  }

  pub fn rm (&self, path: &str) -> Result<()> {
    unsafe {
      if xenstore_sys::xs_rm(self.store.xs, self.tr, CString::new(path).unwrap().as_ptr()) {
        Ok(())
      } else {
        Err(Error::new())
      }
    }
  }

  pub fn commit (&self) -> Result<()> {
    unsafe {
      if xenstore_sys::xs_transaction_end(self.store.xs, self.tr, false) {
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
        xenstore_sys::xs_transaction_end(self.store.xs, self.tr, true);
      }
    }
  }
}
