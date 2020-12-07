use std::ffi::CStr;
use uuid::Uuid;

use super::bindings;
use super::bindings::xc_error_code;

use memmap::Mmap;
use std::fs::File;

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

// -----------------------------------------------------------------------------

pub type CreateDomain = bindings::xen_domctl_createdomain;
pub const XEN_DOMCTL_CDF_HVM: u32 = bindings::XEN_DOMCTL_CDF_hvm;
pub const XEN_DOMCTL_CDF_HAP: u32 = bindings::XEN_DOMCTL_CDF_hap;

pub type ArchDomainConfig = bindings::xen_arch_domainconfig;
pub const XEN_X86_EMU_LAPIC: u32 = bindings::XEN_X86_EMU_LAPIC;

pub type HvmSaveDescriptor = bindings::hvm_save_descriptor;

pub const X86_CR0_PE: u64 = 0x01;
pub const X86_CR0_ET: u64 = 0x10;

pub const X86_DR6_DEFAULT: u64 = 0xffff0ff0;
pub const X86_DR7_DEFAULT: u64 = 0x00000400;

pub const PROT_READ: i32 = libc::PROT_READ;
pub const PROT_WRITE: i32 = libc::PROT_WRITE;

// -----------------------------------------------------------------------------

pub type XenPfn = bindings::xen_pfn_t;

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

  pub fn create_domain (&self, config: &mut CreateDomain) -> Result<u32> {
    unsafe {
      let mut dom_id: u32 = u32::MAX - 1; // let xen choose the dom_id
      match bindings::xc_domain_create(self.xc, &mut dom_id, config) {
        0 => Ok(dom_id),
        _ => Err(self.get_last_error())
      }
    }
  }

  pub fn start_domain (&self, dom_id: u32, image_path: &str) -> Result<()> {
    self.set_max_vcpus_domain(dom_id, 1)?;
    self.set_max_mem_domain(dom_id, u64::MAX)?;

    // Get foreign memory pages
    let length = 16;
    let mut ram: Vec<XenPfn> = vec!();
    ram.resize(length, 0);
    let one_mega: u64 = 1 << 20;
    for i in 0..length {
      ram[i] = (one_mega >> 12) + i as u64
    }
    println!("POPULATE PHYSMAP EXACT DOMAIN");
    self.populate_physmap_exact_domain(dom_id, 0, 0, &mut ram)?;

    println!("FOREIGN MEMORY MAP");
    let ptr = match self.foreign_memory_map(dom_id, PROT_READ | PROT_WRITE, &ram) {
      Ok(ptr) => ptr,
      Err(e) => return Err(e)
    };

    // Open image
    let image = match File::open(image_path) {
      Ok(img) => img,
      Err(_e) => return Err(Error::new(ErrorCode::None, "file open")) // TODO
    };
    unsafe {
      let mmap = match Mmap::map(&image) {
        Ok(mmap) => mmap,
        Err(_e) => return Err(Error::new(ErrorCode::None, "mmap")) // TODO
      };

      // Copy image in foreing pages
      std::ptr::copy_nonoverlapping(mmap.as_ptr(), ptr as *mut u8, mmap.len());
    }

    // 1. Get the HVM context.
    println!("GET HVM CONTEXT {}", dom_id);
    let context = self.get_hvm_context(dom_id)?;
    println!("POPULATE RAM");

    // 2. Create bootstrap context.
    #[derive(Default)]
    struct BootstrapContext {
      header_d: HvmSaveDescriptor,
      header: bindings::HvmSaveTypeHeader,
      cpu_d: HvmSaveDescriptor,
      cpu: bindings::HvmSaveTypeCpu,
      end_d: HvmSaveDescriptor,
      end: bindings::HvmSaveTypeEnd
    };
    let mut bootstrap_buf: Vec<u8> = vec![0; std::mem::size_of::<BootstrapContext>()];
    let mut bootstrap_context = unsafe { std::mem::transmute::<*mut u8, *mut BootstrapContext>(
      bootstrap_buf.as_mut_ptr()
    ) };

    println!("COPY FROM SLICE");
    let bootstrap_size = std::mem::size_of::<HvmSaveDescriptor>() + bindings::HVM_SAVE_LENGTH_HEADER as usize;

    bootstrap_buf[0..bootstrap_size].copy_from_slice(&context[
      0..bootstrap_size
    ]);

    unsafe {
      // 3. Set CPU descriptor.
      (*bootstrap_context).cpu_d.typecode = bindings::HVM_SAVE_CODE_CPU;
      (*bootstrap_context).cpu_d.instance = 0;
      (*bootstrap_context).cpu_d.length = bindings::HVM_SAVE_LENGTH_CPU;

      // 4. Set the cached part of the relevant segment registers.
      (*bootstrap_context).cpu.cs_base = 0;
      (*bootstrap_context).cpu.ds_base = 0;
      (*bootstrap_context).cpu.es_base = 0;
      (*bootstrap_context).cpu.ss_base = 0;
      (*bootstrap_context).cpu.tr_base = 0;
      (*bootstrap_context).cpu.cs_limit = !0;
      (*bootstrap_context).cpu.ds_limit = !0;
      (*bootstrap_context).cpu.es_limit = !0;
      (*bootstrap_context).cpu.ss_limit = !0;
      (*bootstrap_context).cpu.tr_limit = 0x67;
      (*bootstrap_context).cpu.cs_arbytes = 0xc9b;
      (*bootstrap_context).cpu.ds_arbytes = 0xc93;
      (*bootstrap_context).cpu.es_arbytes = 0xc93;
      (*bootstrap_context).cpu.ss_arbytes = 0xc93;
      (*bootstrap_context).cpu.tr_arbytes = 0x8b;

      // 5. Set the control registers.
      (*bootstrap_context).cpu.cr0 = X86_CR0_PE | X86_CR0_ET;

      // 6. Set the GPRs.
      (*bootstrap_context).cpu.rip = one_mega;

      (*bootstrap_context).cpu.dr6 = X86_DR6_DEFAULT;
      (*bootstrap_context).cpu.dr7 = X86_DR7_DEFAULT;

      // TODO: Is it useful?
      // if ( dom->start_info_seg.pfn )
      //     bsp_ctx.cpu.rbx = dom->start_info_seg.pfn << PAGE_SHIFT;

      // 7. Set the end descriptor.
      (*bootstrap_context).end_d.typecode = bindings::HVM_SAVE_CODE_END;
      (*bootstrap_context).end_d.instance = 0;
      (*bootstrap_context).end_d.length = bindings::HVM_SAVE_LENGTH_END;
    }

    // 8. Set context and boot.
    println!("SET HVM CONTEXT");
    self.set_hvm_context(dom_id, &bootstrap_buf)?;
    println!("UNPAUSE DOMAIN");
    self.unpause_domain(dom_id)?;

    println!("UNMAP");
    self.foreign_memory_unmap(ptr, length as u64) // TODO use ram length?
  }

  pub fn get_hvm_context (&self, dom_id: u32) -> Result<Vec<u8>> {
    let mut context = Vec::<u8>::new();
    unsafe {
      println!("GET HVM CONTEXT SIZE");
      let size = bindings::xc_domain_hvm_getcontext(self.xc, dom_id, std::ptr::null_mut(), 0);
      if size <= 0 {
        return Err(self.get_last_error()); // TODO.
      }

      context.resize(size as usize, 0);

      println!("GET HVM CONTEXT BUF");
      if bindings::xc_domain_hvm_getcontext(self.xc, dom_id, context.as_mut_ptr(), size as u32) <= 0 {
        return Err(self.get_last_error()); // TODO.
      }
    }
    println!("RETURN from get context");

    Ok(context)
  }

  pub fn set_hvm_context (&self, dom_id: u32, context: &Vec<u8>) -> Result<()> {
    unsafe {
      match bindings::xc_domain_hvm_setcontext(self.xc, dom_id, context.as_ptr() as *mut u8, context.len() as u32) {
        0 => Ok(()),
        _ => Err(self.get_last_error())
      }
    }
  }

  pub fn set_max_vcpus_domain (&self, dom_id: u32, vcpus: u32) -> Result<()> {
    unsafe {
      match bindings::xc_domain_max_vcpus(self.xc, dom_id, vcpus) {
        0 => Ok(()),
        _ => Err(self.get_last_error())
      }
    }
  }

  pub fn set_max_mem_domain (&self, dom_id: u32, max_mem: u64) -> Result<()> {
    unsafe {
      match bindings::xc_domain_setmaxmem(self.xc, dom_id, max_mem) {
        0 => Ok(()),
        _ => Err(self.get_last_error())
      }
    }
  }

  pub fn populate_physmap_exact_domain (
    &self,
    dom_id: u32,
    extent_order: u32,
    mem_flags: u32,
    extents: &mut Vec<XenPfn>
  ) -> Result<()> {
    unsafe {
      println!("RAM LEN {}", extents.len());
      match bindings::xc_domain_populate_physmap_exact(
        self.xc,
        dom_id,
        extents.len() as u64,
        extent_order,
        mem_flags,
        extents.as_mut_ptr()
      ) {
        0 => Ok(()),
        _ => Err(self.get_last_error())
      }
    }
  }

  pub fn foreign_memory_map (&self, dom_id: u32, prot: i32, arr: &Vec<XenPfn>) -> Result<*mut libc::c_void> {
    unsafe {
      let ret = bindings::xenforeignmemory_map(
        bindings::xc_interface_fmem_handle(self.xc),
        dom_id,
        prot,
        arr.len() as u64,
        arr.as_ptr(),
        std::ptr::null_mut()
      );
      if ret.is_null() {
        return Err(self.get_last_error())
      }

      Ok(ret)
    }
  }

  pub fn foreign_memory_unmap (&self, addr: *mut libc::c_void, size: u64) -> Result<()> {
    unsafe {
      match bindings::xenforeignmemory_unmap(
        bindings::xc_interface_fmem_handle(self.xc),
        addr,
        size
      ) {
        0 => Ok(()),
        _ => Err(self.get_last_error())
      }
    }
  }
}
