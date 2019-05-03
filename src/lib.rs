#[macro_use] extern crate num_derive;
use memmap::Mmap;
use stable_deref_trait::StableDeref;

use std::fs::File;
use std::path::Path;
use std::ops::Deref;


mod core;
mod database; // TODO: move into core

pub mod schema;

type Result<T> = ::std::result::Result<T, Box<std::error::Error>>; // TODO: better error type

pub use crate::core::table::Table;
pub use crate::database::is_database;


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

pub trait TableAccess<'db, T: TableRow> {
    fn get_table(&'db self) -> Table<'db, T::Kind>;
}

macro_rules! impl_table_access {
    ( $tab:ident ) => {
        impl<'db> TableAccess<'db, schema::$tab<'db>> for Database<'db> {
            fn get_table(&'db self) -> Table<'db, <schema::$tab<'db> as TableRow>::Kind> {
                Table {
                    db: self.0.deref(),
                    table: self.0.deref().get_table_info::<<schema::$tab<'db> as TableRow>::Kind>()
                }
            }
        }
    }
}

impl_table_access!(TypeRef);
impl_table_access!(GenericParamConstraint);
impl_table_access!(TypeSpec);
impl_table_access!(TypeDef);
impl_table_access!(CustomAttribute);
impl_table_access!(MethodDef);
impl_table_access!(MemberRef);
impl_table_access!(Module);
impl_table_access!(Param);
impl_table_access!(InterfaceImpl);
impl_table_access!(Constant);
impl_table_access!(Field);
impl_table_access!(FieldMarshal);
impl_table_access!(DeclSecurity);
impl_table_access!(ClassLayout);
impl_table_access!(FieldLayout);
impl_table_access!(StandAloneSig);
impl_table_access!(EventMap);
impl_table_access!(Event);
impl_table_access!(PropertyMap);
impl_table_access!(Property);
impl_table_access!(MethodSemantics);
impl_table_access!(MethodImpl);
impl_table_access!(ModuleRef);
impl_table_access!(ImplMap);
impl_table_access!(FieldRVA);
impl_table_access!(Assembly);
impl_table_access!(AssemblyProcessor);
impl_table_access!(AssemblyOS);
impl_table_access!(AssemblyRef);
impl_table_access!(AssemblyRefProcessor);
impl_table_access!(AssemblyRefOS);
impl_table_access!(File);
impl_table_access!(ExportedType);
impl_table_access!(ManifestResource);
impl_table_access!(NestedClass);
impl_table_access!(GenericParam);
impl_table_access!(MethodSpec);

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

    pub fn table<T: TableRow>(&'db self) -> Table<'db, T::Kind>
        where Self: TableAccess<'db, T>
    {
        self.get_table()
    }
}

pub trait TableRow {
    type Kind: crate::database::TableKind;
}

pub trait TableRowAccess {
    type Table;
    type Out: TableRow;

    fn get(table: Self::Table, row: u32) -> Self::Out;
}
