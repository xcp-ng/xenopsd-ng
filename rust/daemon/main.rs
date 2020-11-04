use enclose::enclose;
use jsonrpc_core::{Error, ErrorCode, IoHandler, Params, Value};
use serde::Deserialize;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};
use xenops::{vm, xenctrl, xenstore};

// =============================================================================

fn make_error (error: &str) -> Error {
  Error { code: ErrorCode::ServerError(0), message: error.to_string(), data: None }
}

// =============================================================================

fn main () {
  let xs = Arc::new(Mutex::new(
    match xenstore::Xenstore::new() {
      Ok(xs) => xs,
      Err(e) => {
        eprintln!("Could not start daemon: {}", e);
        return
      }
    }
  ));

  let xc = Arc::new(Mutex::new(
    match xenctrl::Xenctrl::new() {
      Ok(xc) => xc,
      Err(e) => {
        eprintln!("Could not start daemon: {}", e);
        return
      }
    }
  ));

  let mut io = IoHandler::new();

  io.add_method("host.domain-list", enclose! { (xc) move |_: Params| {
    match xc.lock().unwrap().get_domain_info_list() {
      Ok(domains) => Ok(Value::from_iter(domains.into_iter().map(|dom_info| dom_info.domain))),
      Err(e) => Err(make_error(&e.to_string()))
    }
  } } );

  // See: https://stackoverflow.com/questions/31360003/is-there-another-option-to-share-an-arc-in-multiple-closures-besides-cloning-it
  io.add_method("vm.pause", enclose! { (xc) move |params: Params| {
    #[derive(Deserialize)]
    struct VmPauseParams {
      dom_id: u32
    }

    let parsed: VmPauseParams = params.parse()?;
    match xc.lock().unwrap().pause_domain(parsed.dom_id) {
      Ok(_) => Ok(Value::String(String::from("success"))),
      Err(e) => Err(make_error(&e.to_string()))
    }
  } } );

  io.add_method("vm.unpause", enclose! { (xc) move |params: Params| {
    #[derive(Deserialize)]
    struct VmUnpauseParams {
      dom_id: u32
    }

    let parsed: VmUnpauseParams = params.parse()?;
    match xc.lock().unwrap().unpause_domain(parsed.dom_id) {
      Ok(_) => Ok(Value::String(String::from("success"))),
      Err(e) => Err(make_error(&e.to_string()))
    }
  } } );

  io.add_method("vm.shutdown", enclose! { (xc) move |params: Params| {
    #[derive(Deserialize)]
    struct VmShutdownParams {
      dom_id: u32
    }

    let parsed: VmShutdownParams = params.parse()?;
    match vm::shutdown(&xs.lock().unwrap(), parsed.dom_id, vm::ShutdownReason::PowerOff) {
      Ok(_) => Ok(Value::String(String::from("success"))),
      Err(e) => Err(make_error(&e.to_string()))
    }
  } } );
}
