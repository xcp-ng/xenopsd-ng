use std::env;
use std::ptr::null_mut;
use xenctrl_sys::xc_domain_pause;
use xenctrl_sys::xc_domain_unpause;
use xenctrl_sys::xc_interface;
use xenctrl_sys::xc_interface_close;
use xenctrl_sys::xc_interface_open;

fn help() {
  println!("usage:
xenopsd-ng-cli {{pause|unpause}} <integer>
  pause/unpause a vm if the integer is a valid domid.");
}

unsafe fn pause_vm(xc: *mut xc_interface ,domid: u32) {
  let i = xc_domain_pause(xc, domid);
  if i != 0 {
    eprintln!("error while pausing domain");
  };
}

unsafe fn unpause_vm(xc: *mut xc_interface, domid: u32) {
  let i = xc_domain_unpause(xc, domid);
  if i != 0 {
    eprintln!("error while unpausing domain");
  };
}

fn main() {
  let args: Vec<String> = env::args().collect();

  match args.len() {
    // one command and one argument passed
    3 => {
      let cmd = &args[1];
      let num = &args[2];
      // parse the number
      let domid: u32 = match num.parse() {
        Ok(n) => {
          n
        },
        Err(_) => {
          eprintln!("error: second argument not an integer");
          help();
          return;
        },
      };
      unsafe {
        let xc = xc_interface_open(null_mut(), null_mut(), 0);
        // parse the command
        match &cmd[..] {
          "pause" => pause_vm(xc, domid),
          "unpause" => unpause_vm(xc, domid),
          _ => {
            eprintln!("error: invalid command");
            help();
          },
        };
        let i = xc_interface_close(xc);
        if i != 0 {
          eprintln!("error while closing interface");
        }
      }
    },
    // all the other cases
    _ => {
      help();
    }
  }
}
