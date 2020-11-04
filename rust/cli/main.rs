use std::env;

use xenops::*;

// =============================================================================

fn help () {
  println!("usage:
xenops-cli {{pause|unpause|shutdown}} <integer>
  pause/unpause or shutdown a vm if the integer is a valid domain id.")
}

// -----------------------------------------------------------------------------

fn main () {
  let args: Vec<String> = env::args().collect();

  let xs = match xenstore::Xenstore::new() {
    Ok(xs) => xs,
    Err(e) => {
      eprintln!("Could not execute command: {}", e);
      return
    }
  };
  let xc = match xenctrl::Xenctrl::new() {
    Ok(xc) => xc,
    Err(e) => {
      eprintln!("Could not execute command: {}", e);
      return
    }
  };

  match args.len() {
    // one command passed
    2 => {
      let cmd = &args[1];
      match &cmd[..] {
        "list-domains" => {
          match xc.get_domain_info_list() {
            Ok(domains) => println!("domains: {:?}", domains),
            Err(e) => eprintln!("Error while listing domains: {}", e)
          }
        },
        _ => {
          eprintln!("Error: invalid command");
          help()
        }
      }
    },
    // one command and one argument passed
    3 => {
      let cmd = &args[1];
      let num = &args[2];
      // parse the number
      let dom_id: u32 = match num.parse() {
        Ok(n) => n,
        Err(_) => {
          eprintln!("error: second argument not an integer");
          help();
          return
        }
      };

      // parse the command
      match &cmd[..] {
        "pause" => match xc.pause_domain(dom_id) {
          Ok(_) => (),
          Err(e) => eprintln!("Error while pausing domain: {}, {}", dom_id, e)
        },
        "unpause" => match xc.unpause_domain(dom_id) {
          Ok(_) => (),
          Err(e) => eprintln!("Error while pausing domain: {}, {}", dom_id, e)
        },
        "shutdown" => {
          // TODO: parse from args?
          let reason = vm::ShutdownReason::PowerOff;

          // Useless but demonstrative :)
          // Find uuid from domid
          match xc.get_domain_handle(dom_id) {
            Ok(dom_handle) => println!("Domain UUID {}", xenctrl::get_uuid_from_domain_handle(&dom_handle)),
            Err(e) => eprintln!("Failed to get domain handle: {}", e)
          };

          match vm::shutdown(&xs, dom_id, reason) {
            Ok(()) => println!("Shutdown!"),
            Err(e) => eprintln!("Failed to shutdown: {}", e)
          }
        },
        _ => {
          eprintln!("Error: invalid command");
          help()
        }
      }
    }
    // all the other cases
    _ => help()
  }
}
