#[macro_use] extern crate num_derive;
use memmap::Mmap;
use stable_deref_trait::StableDeref;

use std::fs::File;
use std::path::Path;
use std::ops::Deref;


mod core;

pub mod schema;
pub mod database;

type Result<T> = ::std::result::Result<T, Box<std::error::Error>>; // TODO: better error type


// our own little copy of owning_ref::OwningHandle where the H: Deref bound is dropped
// (see also https://github.com/Kimundi/owning-ref-rs/issues/18 and )
mod owning_ref {
    use std::ops::Deref;
    use stable_deref_trait::StableDeref as StableAddress;

    pub struct OwningHandle<O, H>
        where O: StableAddress,
    {
        handle: H,
        _owner: O,
    }

    impl<O, H> Deref for OwningHandle<O, H>
        where O: StableAddress,
    {
        type Target = H;
        fn deref(&self) -> &H {
            &self.handle
        }
    }

    impl<O, H> OwningHandle<O, H>
        where O: StableAddress,
    {
        /// Create a new OwningHandle. The provided callback will be invoked with
        /// a pointer to the object owned by `o`, and the returned value is stored
        /// as the object to which this `OwningHandle` will forward `Deref` and
        /// `DerefMut`.
        pub fn try_new<F, E>(o: O, f: F) -> Result<Self, E>
            where F: FnOnce(*const O::Target) -> Result<H, E>
        {
            let h: H;
            {
                h = f(o.deref() as *const O::Target)?;
            }

            Ok(OwningHandle {
            handle: h,
            _owner: o,
            })
        }
    }
}

use self::owning_ref::OwningHandle;

struct StableMmap(Mmap);

impl Deref for StableMmap {
    type Target = [u8];
    fn deref(&self) -> &Self::Target { &self.0 }
}

// The Deref result for Mmap does not depend on the actual location of the
// Mmap object, but solely on the mapped memory, so this is safe
unsafe impl StableDeref for StableMmap {}

// Separate type because enum variants are always public
enum DatabaseInner<'db> {
    Owned(OwningHandle<StableMmap, database::Database<'db>>),
    Borrowed(database::Database<'db>)
}

impl<'db> Deref for DatabaseInner<'db> {
    type Target = database::Database<'db>;
    fn deref(&self) -> &Self::Target {
        use DatabaseInner::*;
        match self {
            Owned(ref handle) => handle.deref(),
            Borrowed(ref db) => db
        }
    }
}


pub struct Database<'db>(DatabaseInner<'db>);

impl<'db> Database<'db> {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Database<'db>> {
        let file = File::open(path.as_ref())?;
        let mmap = StableMmap(unsafe { Mmap::map(&file)? });
        Ok(Database(DatabaseInner::Owned(
            OwningHandle::try_new(mmap, |ptr: *const [u8]| unsafe { database::Database::load(&(*ptr)[..]) })?
        )))
    }

    pub fn from_data<'a>(data: &'a [u8]) -> Result<Database<'a>> {
        Ok(Database(DatabaseInner::Borrowed(database::Database::load(data)?)))
    }

    pub fn get_table<T: crate::schema::TableRow>(&'db self) -> crate::core::table::Table<'db, T::Kind>
        where database::Database<'db>: database::TableAccess<'db, T::Kind>
    {
        crate::core::table::Table {
            db: self.0.deref(),
            table: self.0.deref().get_table_info::<T::Kind>()
        }
    }
}
