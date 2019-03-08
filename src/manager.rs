use std::fmt;
use std::fs::File;
use std::io::{self, Read};
pub use warmy::{Load, Loaded, Storage};
pub use warmy::{Store, StoreOpt};

use std::fmt::Display;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SimpleKey {
  /// A key to a resource living on the filesystem.
  Path(PathBuf),
  /// A key to a resource living in memory or computed on the fly.
  Logical(PathBuf),
}

impl SimpleKey {
  pub fn from_path<P>(path: P) -> Self
  where
    P: AsRef<Path>,
  {
    SimpleKey::Path(path.as_ref().to_owned())
  }
}

impl<'a> From<&'a Path> for SimpleKey {
  fn from(path: &Path) -> Self {
    match path.extension().and_then(|x| x.to_str()) {
      None => SimpleKey::from_path(path),
      Some("hlsl") => SimpleKey::Logical(path.to_path_buf()),
      _ => SimpleKey::from_path(path),
    }
  }
}

impl From<PathBuf> for SimpleKey {
  fn from(path: PathBuf) -> Self {
    SimpleKey::Path(path)
  }
}

impl Into<Option<PathBuf>> for SimpleKey {
  fn into(self) -> Option<PathBuf> {
    match self {
      SimpleKey::Path(path) => Some(path),
      SimpleKey::Logical(path) => Some(path),
    }
  }
}

impl Display for SimpleKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      SimpleKey::Path(ref path) => write!(f, "{}", path.display()),
      SimpleKey::Logical(ref name) => write!(f, "{}", name.display()),
    }
  }
}

impl warmy::Key for SimpleKey {
  fn prepare_key(self, root: &Path) -> Self {
    match self {
      SimpleKey::Path(path) => SimpleKey::Path(vfs_substitute_path(&path, root)),
      SimpleKey::Logical(path) => SimpleKey::Logical(vfs_substitute_path(&path, root)),
    }
  }
}
/// Substitute a VFS path by a real one.
fn vfs_substitute_path(path: &Path, root: &Path) -> PathBuf {
  let mut components = path.components().peekable();
  let root_components = root.components();

  match components.peek() {
    Some(&Component::RootDir) => {
      // drop the root component
      root_components.chain(components.skip(1)).collect()
    }

    _ => root_components.chain(components).collect(),
  }
}

// Possible errors that might happen.
#[derive(Debug)]
pub enum Error {
  CannotLoadFromFS,
  CannotLoadFromLogical,
  IOError(io::Error),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Error::CannotLoadFromFS => f.write_str("cannot load from file system"),
      Error::CannotLoadFromLogical => f.write_str("cannot load from logical"),
      Error::IOError(ref e) => write!(f, "IO error: {}", e),
    }
  }
}

// The resource we want to take from a file.
#[derive(Debug, Clone)]
pub struct FromFS {
  pub bytes: Vec<u8>,
  pub version: u8,
}

// The resource we want to compute from memory.
#[derive(Debug)]
pub struct ShaderSet {
  pub version: u8,
  pub vx: Vec<u8>,
  pub px: Vec<u8>,
}

pub struct Ctx {}

impl Ctx {
  pub fn new() -> Self {
    Ctx {}
  }
}

impl Load<Ctx, SimpleKey> for FromFS {
  type Error = Error;

  fn load(
    key: SimpleKey,
    storage: &mut Storage<Ctx, SimpleKey>,
    ctx: &mut Ctx,
  ) -> Result<Loaded<Self, SimpleKey>, Self::Error> {
    // as we only accept filesystem here, we’ll ensure the key is a filesystem one
    match key {
      SimpleKey::Path(path) => {
        println!("Load Physical {}", path.display());
        let mut fh = File::open(path).map_err(Error::IOError)?;
        let mut buf = Vec::default();
        fh.read_to_end(&mut buf).expect("Load failed");
        // storage.get::<ShaderSet>(&dep, ctx).unwrap();
        Ok(Loaded::without_dep(
          FromFS {
            bytes: buf,
            version: 1,
          }
          .into()
        ))
      }

      SimpleKey::Logical(_) => Err(Error::CannotLoadFromLogical),
    }
  }
  fn reload(
    &self,
    key: SimpleKey,
    storage: &mut Storage<Ctx, SimpleKey>,
    ctx: &mut Ctx,
  ) -> Result<Self, Error> {
    let prev = storage.get_by::<FromFS, AlwaysFail>(&key, ctx, AlwaysFail);
    let prev_version = prev.map(|x| x.borrow().version).unwrap_or(0);
    let l: Result<Loaded<Self, SimpleKey>, Error> =
      <FromFS as warmy::load::Load<Ctx, SimpleKey, ()>>::load(key, storage, ctx);
    l.map(|mut lr| {
      lr.res.version = prev_version + 1;
      println!("  new version {}", lr.res.version);
      lr.res
    })
  }
}

struct AlwaysFail;
impl Load<Ctx, SimpleKey, AlwaysFail> for FromFS {
  type Error = Error;

  fn load(
    _key: SimpleKey,
    _storage: &mut Storage<Ctx, SimpleKey>,
    _ctx: &mut Ctx,
  ) -> Result<Loaded<Self, SimpleKey>, Self::Error> {
    Err(Error::CannotLoadFromFS)
  }
}

impl Load<Ctx, SimpleKey, AlwaysFail> for ShaderSet {
  type Error = Error;

  fn load(
    _key: SimpleKey,
    _storage: &mut Storage<Ctx, SimpleKey>,
    _ctx: &mut Ctx,
  ) -> Result<Loaded<Self, SimpleKey>, Self::Error> {
    Err(Error::CannotLoadFromFS)
  }
}

impl Load<Ctx, SimpleKey> for ShaderSet {
  type Error = Error;

  fn load(
    key: SimpleKey,
    _storage: &mut Storage<Ctx, SimpleKey>,
    _ctx: &mut Ctx,
  ) -> Result<Loaded<Self, SimpleKey>, Error> {
    match key {
      SimpleKey::Logical(key) => {
        println!("Load logical {}", key.display());
        
        use std::process::Command;
        use std::io::Write;

        let output = Command::new("cmd")
          .args(&["/C", "compile.cmd"])
          .output()
          .expect("failed to execute process");

        println!("Shader compilation status: {}", output.status);
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();


        let mut fh = File::open("data/vertex.fx").map_err(Error::IOError)?;
        let mut vx = Vec::default();
        fh.read_to_end(&mut vx).expect("Load failed");

        let mut fh = File::open("data/pixel.fx").map_err(Error::IOError)?;
        let mut px = Vec::default();
        fh.read_to_end(&mut px).expect("Load failed");
          
        Ok(Loaded::without_dep(ShaderSet {
          version: 1,
          vx: vx,
          px: px,
        }))
      }

      SimpleKey::Path(_) => Err(Error::CannotLoadFromFS),
    }
  }

  fn reload(
    &self,
    key: SimpleKey,
    storage: &mut Storage<Ctx, SimpleKey>,
    ctx: &mut Ctx,
  ) -> Result<Self, Error> {
    println!("reload shader set");
    let prev = storage.get_by::<ShaderSet, AlwaysFail>(&key, ctx, AlwaysFail);
    let prev_version = prev.map(|x| x.borrow().version).unwrap_or(0);
    let l: Result<Loaded<Self, SimpleKey>, Error> =
      <ShaderSet as warmy::load::Load<Ctx, SimpleKey, ()>>::load(key, storage, ctx);
    l.map(|mut lr| {
      lr.res.version = prev_version + 1;
      println!("  new version {}", lr.res.version);
      lr.res
    })
  }
}

pub type ResourceManager = Store<Ctx, SimpleKey>;
