use super::xenstore;

// =============================================================================

pub enum ShutdownReason {
  PowerOff,
  Reboot,
  Suspend,
  Crash,
  Halt,
  S3Suspend,
  Unknown(i32)
}

impl std::fmt::Display for ShutdownReason {
  fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match *self {
      ShutdownReason::PowerOff => write!(f, "poweroff"),
      ShutdownReason::Reboot => write!(f, "reboot"),
      ShutdownReason::Suspend => write!(f, "suspend"),
      ShutdownReason::Crash => write!(f, "crash"),
      ShutdownReason::Halt => write!(f, "halt"),
      ShutdownReason::S3Suspend => write!(f, "s3"),
      ShutdownReason::Unknown(code) => write!(f, "(unknown {})", code)
    }
  }
}

pub fn shutdown (xs: &xenstore::Xenstore, dom_id: u32, reason: ShutdownReason) -> xenstore::Result<()> {
  let domain_path = xs.get_domain_path(dom_id);
  let shutdown_path = domain_path.clone() + "/control/shutdown";

  println!("{}", domain_path);

  let transaction = xenstore::Transaction::new(&xs)?;

  transaction.read(&domain_path)?;
  let _ = transaction.rm(&shutdown_path);
  transaction.write(&shutdown_path, &reason.to_string())?;
  transaction.commit()
}

pub fn get_name (xs: &xenstore::Xenstore, dom_id: u32) -> xenstore::Result<String> {
  xs.read(&(xs.get_domain_path(dom_id) + "/name"))
}
