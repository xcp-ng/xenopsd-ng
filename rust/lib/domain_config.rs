use ini::Ini;

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

// -----------------------------------------------------------------------------

pub type Result<T> = std::result::Result<T, Error>;

// =============================================================================

pub enum DomainType {
  Hvm,
  PvHvm,
  Pv
}

pub struct Config {
  name: String,
  vcpus: u32,
  domain_type: DomainType,
  memory: usize,
  kernel: String,
  firmware_override: String
}

impl Config {
  pub fn new () -> Self {
    Self {
      name: String::new(),
      vcpus: 0,
      domain_type: DomainType::Pv,
      memory: 0,
      kernel: String::new(),
      firmware_override: String::new()
    }
  }

  pub fn read_from_path (&self, path: &str) -> Result<()> {
    let i = Ini::load_from_file(path).unwrap();
    for (sec, prop) in i.iter() {
      println!("Section: {:?}", sec);
      for (k, v) in prop.iter() {
        println!("{}:{}", k, v);
      }
    }
    Ok(())
  }
}

