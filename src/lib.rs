#[macro_use] extern crate num_derive;
use memmap::Mmap;
use stable_deref_trait::StableDeref;
use owning_ref::OwningHandle;
use elsa::FrozenVec;

use std::fs::File;
use std::path::Path;
use std::ops::Deref;
use std::collections::HashMap;
use std::cell::RefCell;

mod core;

use crate::core::db;

pub mod schema;

type Result<T> = ::std::result::Result<T, Box<std::error::Error>>; // TODO: better error type

pub use crate::core::table::Table;
pub use db::is_database;

struct StableMmap(Mmap);

impl Deref for StableMmap {
    type Target = [u8];
    fn deref(&self) -> &Self::Target { &self.0 }
}

// The Deref result for Mmap does not depend on the actual location of the
// Mmap object, but solely on the mapped memory, so this is safe
unsafe impl StableDeref for StableMmap {}

// OwningHandle requires the Handle to implement Deref (see also https://github.com/Kimundi/owning-ref-rs/issues/18)
struct DerefDatabase<'db>(db::Database<'db>);

impl<'db> Deref for DerefDatabase<'db> {
    type Target = db::Database<'db>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

// Separate type because enum variants are always public
enum DatabaseInner<'db> {
    Owned(OwningHandle<StableMmap, DerefDatabase<'db>>),
    Borrowed(db::Database<'db>)
}

impl<'db> Deref for DatabaseInner<'db> {
    type Target = db::Database<'db>;
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
            OwningHandle::try_new(mmap, |ptr: *const [u8]| db::Database::load(unsafe { &(*ptr)[..] }).map(|db| DerefDatabase(db)))?
        )))
    }

    pub fn from_data<'a>(data: &'a [u8]) -> Result<Database<'a>> {
        Ok(Database(DatabaseInner::Borrowed(db::Database::load(data)?)))
    }

    pub fn table<T: TableRow>(&'db self) -> Table<'db, T::Kind>
        where Self: TableAccess<'db, T>
    {
        self.get_table()
    }
}

pub trait TableRow {
    type Kind: db::TableKind;
}

pub trait TableRowAccess {
    type Table;
    type Out: TableRow;

    fn get(table: Self::Table, row: u32) -> Self::Out;
}

#[derive(Default)]
pub struct Cache<'db> {
    databases: FrozenVec<Box<Database<'db>>>,
    namespace_map: RefCell<HashMap<&'db str, MemberCache<'db>>>
}

impl<'db> Cache<'db> {
    pub fn new() -> Cache<'db> {
        Cache {
            databases: FrozenVec::new(),
            namespace_map: RefCell::new(HashMap::new())
        }
    }

    pub fn insert(&'db self, database: Database<'db>) -> &'db Database<'db> {
        use std::collections::hash_map::Entry::*;

        let idx = self.databases.len();
        self.databases.push(Box::new(database));
        let db = &self.databases[idx];
        for typ in db.table::<schema::TypeDef>() {
            // if !type.flags().windows_runtime() {
            //     continue;
            // }

            let mut map = self.namespace_map.borrow_mut();
            let members = map.entry(typ.type_namespace().expect("unable to read namespace")).or_insert(MemberCache::default());
            match members.types.entry(typ.type_name().expect("unable to read type name")) {
                Occupied(_) => {},
                Vacant(e) => { e.insert(typ.clone()); }
            }
        }
        db
    }

    pub fn find(&'db self, type_namespace: &str, type_name: &str) -> Option<schema::TypeDef<'db>> {
        let map = self.namespace_map.borrow();
        map.get(type_namespace).and_then(|ns| ns.types.get(type_name).map(|t| t.clone()))
    }

    pub fn iter(&'db self) -> elsa::vec::Iter<'db, Box<Database<'db>>> {
        self.into_iter()
    }
}

impl<'db> IntoIterator for &'db Cache<'db> {
    type Item = &'db Database<'db>;
    type IntoIter = elsa::vec::Iter<'db, Box<Database<'db>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.databases.into_iter()
    }
}

#[derive(Default)]
struct MemberCache<'db> {
    types: HashMap<&'db str, schema::TypeDef<'db>>,
}


pub trait ResolveToTypeDef<'db> {
    fn resolve(&self, cache: &'db Cache<'db>) -> Option<schema::TypeDef<'db>>;
}

impl<'db> ResolveToTypeDef<'db> for &str {
    fn resolve(&self, cache: &'db Cache<'db>) -> Option<schema::TypeDef<'db>> {
        let (namespace, name) = match self.rfind('.') {
            None => return None,
            Some(dot) => (&self[..dot], &self[dot+1 ..])
        };
        cache.find(namespace, name)
    }
}
