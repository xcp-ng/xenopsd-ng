use std::env;

mod xenctrl;

fn help() {
  println!("usage:
cargo run {{pause|unpause}} <integer>
  pause/unpause a vm if the integer is a valid domid.");
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
        Ok(n) => n,
        Err(_) => {
          eprintln!("error: second argument not an integer");
          help();
          return;
        },
      };

      // parse the command
      let xc = match xenctrl::Xenctrl::new() {
        Ok(n) => n,
        Err(e) => {
          eprintln!("Error while opening xenctrl interface: {:?}", e);
          return;
        }
      };
      match &cmd[..] {
        "pause" => match xc.pause_domain(domid) {
          Ok(_) => (),
          Err(e) => eprintln!("Error while pausing domain: {}, {}", domid, e)
        },
        "unpause" => match xc.unpause_domain(domid) {
          Ok(_) => (),
          Err(e) => eprintln!("Error while pausing domain: {}, {}", domid, e)
        },
        _ => {
          eprintln!("error: invalid command");
          help();
        },
      };
    },
    // all the other cases
    _ => help()
  }
}
