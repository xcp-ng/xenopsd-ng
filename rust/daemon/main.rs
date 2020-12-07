use enclose::enclose;
use jsonrpc_core::{Error, ErrorCode, IoHandler, Params, Value, serde_json::json};
use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation, RestApi, ServerBuilder};
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

  io.add_method("host.domain-list", enclose! { (xc, xs) move |_: Params| {
    let xs = xs.lock().unwrap();
    match xc.lock().unwrap().get_domain_info_list() {
      Ok(domains) => Ok(Value::from_iter(domains.into_iter().map(|dom_info| {
        let name = match vm::get_name(&xs, dom_info.domain.into()) {
          Ok(vm_name) => vm_name,
          Err(_) => String::from("(null)")
        };
        json!({
          "dom_id": dom_info.domain,
          "name": name
        })
      }))),
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

  io.add_method("vm.shutdown", enclose! { (xs) move |params: Params| {
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

  io.add_method("vm.create", enclose! { (xc) move |params: Params| {
    #[derive(Deserialize)]
    struct VmShutdownParams {
      image_path: String
    }

    let parsed: VmShutdownParams = params.parse()?;
    let create_domain = &mut xenctrl::CreateDomain {
      flags: xenctrl::XEN_DOMCTL_CDF_HVM | xenctrl::XEN_DOMCTL_CDF_HAP,
      max_vcpus: 1,
      max_evtchn_port: u32::MAX, // -1 as u32
      max_grant_frames: 64,
      max_maptrack_frames: 1024,
      arch: xenctrl::ArchDomainConfig {
        emulation_flags: xenctrl::XEN_X86_EMU_LAPIC
      },
      handle: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      iommu_opts: 0,
      ssidref: 0
    };
    let dom_id = match xc.lock().unwrap().create_domain(create_domain) {
      Ok(v) => v,
      Err(e) => return Err(make_error(&e.to_string()))
    };

    match xc.lock().unwrap().start_domain(dom_id, &parsed.image_path) {
      Ok(_) => Ok(json!(dom_id)),
      Err(e) => Err(make_error(&e.to_string()))
    }
  } } );

  let server = ServerBuilder::new(io)
    .threads(2)
    .rest_api(RestApi::Unsecure)
    .cors(DomainsValidation::AllowOnly(vec![AccessControlAllowOrigin::Any]))
    .start_http(&"0.0.0.0:3030".parse().unwrap()) // Any ip.
    .expect("Unable to start RPC server");

  server.wait();
}
