use std::ffi::CStr;
use uuid::Uuid;

use super::bindings;
use super::bindings::xc_error_code;

// =============================================================================

macro_rules! DefErrorCode {
  ($($name:ident: $c_value:path)*) => {
    #[derive(PartialEq)]
    pub enum ErrorCode {
      $($name, )*
      OsError(i32)
    }

    impl ErrorCode {
      pub fn value (&self) -> i32 {
        match *self {
          $(Self::$name => $c_value as i32, )*
          Self::OsError(code) => -code,
        }
      }

      // Internal helper to build ErrorCode from C.
      fn from_c (value: xc_error_code) -> Self {
        match value {
          $($c_value => Self::$name, )*
        }
      }
    }
  }
}

DefErrorCode! {
  None: xc_error_code::XC_ERROR_NONE
  InternalError: xc_error_code::XC_INTERNAL_ERROR
  InvalidKernel: xc_error_code::XC_INVALID_KERNEL
  InvalidParam: xc_error_code::XC_INVALID_PARAM
  OutOfMemory: xc_error_code::XC_OUT_OF_MEMORY
}

// -----------------------------------------------------------------------------

pub struct Error {
  code: ErrorCode,
  details: String
}

impl Error {
  pub fn new (code: ErrorCode, details: &str) -> Self {
    Self { code, details: details.to_string() }
  }

  pub fn empty () -> Self {
    Self {
      code: ErrorCode::None,
      details: String::new()
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let value = self.code.value();
    let str = match value {
      0 => String::from("Empty error"),
      n if n > 0 => {
        unsafe {
          let description = bindings::xc_error_code_to_desc(n);
          if description.is_null() {
            String::from("Unknown error")
          } else {
            String::from(CStr::from_ptr(description).to_str().unwrap())
          }
        }
      }
      n => std::io::Error::from_raw_os_error(-n).to_string()
    };

    write!(f, "{}: {} ({})", value, str, self.details)
  }
}

pub type Result<T> = std::result::Result<T, Error>;

// -----------------------------------------------------------------------------

type DomainHandle = bindings::xen_domain_handle_t;

pub fn get_uuid_from_domain_handle (dom_handle: &DomainHandle) -> String {
  Uuid::from_bytes(dom_handle).unwrap().to_string()
}

// -----------------------------------------------------------------------------

pub type DomainInfo = bindings::xen_domctl_getdomaininfo_t;

// =============================================================================

pub struct Xenctrl {
  xc: *mut bindings::xc_interface
}

unsafe impl Send for Xenctrl {}

impl Drop for Xenctrl {
  fn drop (&mut self) {
    unsafe { bindings::xc_interface_close(self.xc); }
  }
}

impl Xenctrl {
  pub fn new () -> std::result::Result<Self, &'static str> {
    unsafe {
      let xc = bindings::xc_interface_open(std::ptr::null_mut(), std::ptr::null_mut(), 0);
      if !xc.is_null() { Ok(Self { xc }) } else { Err("Failed to open xenctrl interface") }
    }
  }

  pub fn get_domain_info (&self, dom_id: u32) -> Result<DomainInfo> {
    unsafe {
      let mut info: DomainInfo = std::mem::MaybeUninit::uninit().assume_init();
      let ret = bindings::xc_domain_getinfolist(self.xc, dom_id, 1, &mut info);
      if ret != 1 || u32::from(info.domain) != dom_id {
        let error = self.get_last_error();
        if error.code == ErrorCode::None {
          return Err(Error::new(ErrorCode::InvalidParam, ""))
        } else {
          return Err(error)
        }
      }

      Ok(info)
    }
  }

  pub fn get_domain_info_list (&self) -> Result<Vec<DomainInfo>> {
    unsafe {
      let max_doms: u32 = 1024;
      let mut chunk: Vec<DomainInfo> = Vec::with_capacity(max_doms as usize);
      chunk.resize_with(max_doms as usize, Default::default);
      let mut dom_id = 0;
      let mut domains = Vec::new();
      loop {
        let ret = bindings::xc_domain_getinfolist(self.xc, dom_id, max_doms, chunk.as_mut_ptr());
        match ret {
          -1 => {
            let error = self.get_last_error();
            if error.code == ErrorCode::None {
              return Err(Error::new(ErrorCode::InvalidParam, ""))
            } else {
              return Err(error)
            }
          },
          0 => break,
          n => {
            let n = n as usize;
            domains.reserve(n);
            for i in 0..n {
              let dom_info = chunk[i];
              let info_dom_id = dom_info.domain;
              dom_id = std::cmp::max(dom_id, info_dom_id.into()) + 1;
              domains.push(dom_info);
            }
          }
        }
      }

      Ok(domains)
    }
  }

  pub fn get_last_error (&self) -> Error {
    unsafe {
      let error = bindings::xc_get_last_error(self.xc);
      match ErrorCode::from_c((*error).code) {
        ErrorCode::None => {
          let os_error = std::io::Error::last_os_error().raw_os_error().unwrap();
          if os_error != 0 {
            Error::new(ErrorCode::OsError(os_error), "")
          } else {
            Error::empty()
          }
        },
        code => {
          let details: String;
          if (*error).message.is_empty() {
            details = String::new();
          } else {
            details = String::from(CStr::from_ptr((*error).message.as_ptr()).to_str().unwrap());
          }
          Error::new(code, &details)
        }
      }
    }
  }

  pub fn pause_domain (&self, dom_id: u32) -> Result<()> {
    unsafe {
      match bindings::xc_domain_pause(self.xc, dom_id) {
        0 => Ok(()),
        _ => Err(self.get_last_error())
      }
    }
  }

  pub fn unpause_domain (&self, dom_id: u32) -> Result<()> {
    unsafe {
      match bindings::xc_domain_unpause(self.xc, dom_id) {
        0 => Ok(()),
        _ => Err(self.get_last_error())
      }
    }
  }
}
