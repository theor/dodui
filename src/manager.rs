use std::fmt;
use std::fs::File;
use std::io::{self, Read};
pub use warmy::{Load, Loaded, SimpleKey, Storage};
pub use warmy::{Store, StoreOpt};

// pub struct Id(u32);

// pub trait Resource {}

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
#[derive(Debug)]
pub struct FromFS(pub Vec<u8>);

// The resource we want to compute from memory.
#[derive(Debug)]
pub struct ShaderSet {
    version: u8,
    vs: FromFS,
    ps: FromFS,
}

pub struct Ctx {
    // f: Arc<F>,
}

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
        _: &mut Ctx,
    ) -> Result<Loaded<Self, SimpleKey>, Self::Error> {
        // as we only accept filesystem here, we’ll ensure the key is a filesystem one
        match key {
            SimpleKey::Path(path) => {
                println!("Load Physical {:?}", path);
                let mut fh = File::open(path).map_err(Error::IOError)?;
                let mut buf = Vec::default();
                fh.read_to_end(&mut buf);

                Ok(FromFS(buf).into())
            }

            SimpleKey::Logical(_) => Err(Error::CannotLoadFromLogical),
        }
    }
}

struct AlwaysFail;
impl Load<Ctx, SimpleKey, AlwaysFail> for ShaderSet {
    type Error = Error;

    fn load(
        key: SimpleKey,
        storage: &mut Storage<Ctx, SimpleKey>,
        ctx: &mut Ctx,
    ) -> Result<Loaded<Self, SimpleKey>, Self::Error> {
        Err(Error::CannotLoadFromFS)
    }
}

impl Load<Ctx, SimpleKey> for ShaderSet {
    type Error = Error;

    fn load(
        key: SimpleKey,
        storage: &mut Storage<Ctx, SimpleKey>,
        ctx: &mut Ctx,
    ) -> Result<Loaded<Self, SimpleKey>, Error> {
        // ensure we only accept logical resources
        match key {
            SimpleKey::Logical(key) => {
                println!("Load logical {}", key);
                use std::path::Path;
                let vk = Path::new("data/vertex.fx").into();
                let pk = Path::new("data/pixel.fx").into();
                let vs = storage.get::<FromFS>(&vk, ctx).unwrap();
                let ps = storage.get::<FromFS>(&pk, ctx).unwrap();
                // use gfx::traits::FactoryExt;
                // ctx.f
                //     .create_pipeline_simple(
                //         &(*vs.borrow()).0,
                //         &(*ps.borrow()).0,
                //         crate::rendering::pipe::new(),
                //     )
                //     .unwrap();
                Ok(Loaded::with_deps(ShaderSet {
                    version: 0,
                    vs: *vs.borrow(),
                    ps: *ps.borrow(),
                }, vec![vk, pk]))
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
        let prev = storage.get_by::<ShaderSet, AlwaysFail>(&key, ctx, AlwaysFail);
        let prev_version = prev.map(|x| x.borrow().version).unwrap_or(0);
        let l: Result<Loaded<Self, SimpleKey>, Error> = <ShaderSet as warmy::load::Load<Ctx, SimpleKey, ()>>::load(key, storage, ctx);
        l.map(|mut lr| { lr.res.version = prev_version + 1; println!("  new version {}", lr.res.version); lr.res })
    }
}

pub type ResourceManager = Store<Ctx, SimpleKey>;
