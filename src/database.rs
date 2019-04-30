use std::fs::File;
use std::path::Path;
use std::mem;
use std::io;

use byteorder::{ByteOrder, LittleEndian};
use memmap::{MmapOptions, Mmap};

use crate::{pe, schema};
use crate::table::Column;
use crate::Result;
use crate::ByteView;

pub use crate::table::{Table, TableRow, TableRowIterator};

pub(crate) trait ReadValue<S: ColumnSize> {
    fn read_value(input: &[u8], size: u8) -> Self;
}

impl ReadValue<FixedSize2> for u16 {
    fn read_value(input: &[u8], _: u8) -> Self {
        LittleEndian::read_u16(input)
    }
}

impl ReadValue<FixedSize4> for u32 {
    fn read_value(input: &[u8], _: u8) -> Self {
        LittleEndian::read_u32(input)
    }
}

impl ReadValue<FixedSize8> for u64 {
    fn read_value(input: &[u8], _: u8) -> Self {
        LittleEndian::read_u64(input)
    }
}

impl ReadValue<DynamicSize> for u32 {
    fn read_value(input: &[u8], size: u8) -> Self {
        if size == 4 {
            LittleEndian::read_u32(input)
        } else {
            LittleEndian::read_u16(input) as u32
        }
    }
}

pub(crate) trait ColumnIndex { fn idx() -> usize; }

pub(crate) struct Col0;
impl ColumnIndex for Col0 { fn idx() -> usize { 0 } }
pub(crate) struct Col1;
impl ColumnIndex for Col1 { fn idx() -> usize { 1 } }
pub(crate) struct Col2;
impl ColumnIndex for Col2 { fn idx() -> usize { 2 } }
pub(crate) struct Col3;
impl ColumnIndex for Col3 { fn idx() -> usize { 3 } }
pub(crate) struct Col4;
impl ColumnIndex for Col4 { fn idx() -> usize { 4 } }
pub(crate) struct Col5;
impl ColumnIndex for Col5 { fn idx() -> usize { 5 } }

pub(crate) trait ColumnTuple: Copy {
    fn row_size(&self) -> u8;
    fn init(&self, cols: &mut [Column]);
}

pub(crate) trait ColumnTupleAccess<Col: ColumnIndex>: ColumnTuple {
    type Out: ColumnSize;
}

impl<C0: ColumnSize> ColumnTuple for (C0,) {
    fn row_size(&self) -> u8 { self.0.size() }
    fn init(&self, cols: &mut [Column]) { cols[0] = Column { offset: 0, size: self.0.size() }; }
}
impl<C0: ColumnSize> ColumnTupleAccess<Col0> for (C0,) { type Out = C0; }

impl<C0: ColumnSize, C1: ColumnSize> ColumnTuple for (C0, C1) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() }
    fn init(&self, cols: &mut [Column]) { (self.0,).init(cols); cols[1] = Column { offset: cols[0].offset + cols[0].size, size: self.1.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1) { type Out = C1; }

impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize> ColumnTuple for (C0, C1, C2) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() + self.2.size() }
    fn init(&self, cols: &mut [Column]) { (self.0, self.1).init(cols); cols[2] = Column { offset: cols[1].offset + cols[1].size, size: self.2.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1, C2) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1, C2) { type Out = C1; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize> ColumnTupleAccess<Col2> for (C0, C1, C2) { type Out = C2; }

impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTuple for (C0, C1, C2, C3) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() + self.2.size() + self.3.size() }
    fn init(&self, cols: &mut [Column]) { (self.0, self.1, self.2).init(cols); cols[3] = Column { offset: cols[2].offset + cols[2].size, size: self.3.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1, C2, C3) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1, C2, C3) { type Out = C1; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTupleAccess<Col2> for (C0, C1, C2, C3) { type Out = C2; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTupleAccess<Col3> for (C0, C1, C2, C3) { type Out = C3; }

impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTuple for (C0, C1, C2, C3, C4) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() + self.2.size() + self.3.size() + self.4.size() }
    fn init(&self, cols: &mut [Column]) { (self.0, self.1, self.2, self.3).init(cols); cols[4] = Column { offset: cols[3].offset + cols[3].size, size: self.4.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1, C2, C3, C4) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1, C2, C3, C4) { type Out = C1; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col2> for (C0, C1, C2, C3, C4) { type Out = C2; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col3> for (C0, C1, C2, C3, C4) { type Out = C3; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col4> for (C0, C1, C2, C3, C4) { type Out = C4; }

impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTuple for (C0, C1, C2, C3, C4, C5) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() + self.2.size() + self.3.size() + self.4.size() + self.5.size() }
    fn init(&self, cols: &mut [Column]) { (self.0, self.1, self.2, self.3, self.4).init(cols); cols[5] = Column { offset: cols[4].offset + cols[4].size, size: self.5.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1, C2, C3, C4, C5) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1, C2, C3, C4, C5) { type Out = C1; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col2> for (C0, C1, C2, C3, C4, C5) { type Out = C2; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col3> for (C0, C1, C2, C3, C4, C5) { type Out = C3; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col4> for (C0, C1, C2, C3, C4, C5) { type Out = C4; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col5> for (C0, C1, C2, C3, C4, C5) { type Out = C5; }

pub(crate) trait ColumnSize: Copy {
    fn size(&self) -> u8;
}

#[derive(Copy, Clone, Default)]
pub(crate) struct FixedSize2;
impl ColumnSize for FixedSize2 {
    fn size(&self) -> u8 { 2 }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct FixedSize4;
impl ColumnSize for FixedSize4 {
    fn size(&self) -> u8 { 4 }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct FixedSize8;
impl ColumnSize for FixedSize8 {
    fn size(&self) -> u8 { 8 }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum DynamicSize {
    Unset,
    Size2,
    Size4
}
impl ColumnSize for DynamicSize {
    fn size(&self) -> u8 {
        match *self {
            DynamicSize::Unset => panic!("uninitialized dynamic column"),
            DynamicSize::Size2 => 2,
            DynamicSize::Size4 => 4
        }
    }
}
impl Default for DynamicSize {
    fn default() -> Self {
        DynamicSize::Unset
    }
}

pub trait TableKind: Copy {}

pub(crate) trait TableDesc: TableKind {
    type Columns;
}

pub trait TableAccess<'db, T> {
    fn get_table_info(&self) -> &TableInfo<'db, T>;
}

macro_rules! impl_table_access {
    ( $tab:ident ) => {
        impl<'db> TableAccess<'db, schema::$tab> for Tables<'db> {
            fn get_table_info(&self) -> &TableInfo<'db, schema::$tab> {
                &self.$tab
            }
        }

        impl<'db> TableAccess<'db, schema::$tab> for Database<'db> {
            fn get_table_info(&self) -> &TableInfo<'db, schema::$tab> {
                &self.m_tables.$tab
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


// TODO: not pub?
pub struct TableInfo<'db, T> {
    p: std::marker::PhantomData<T>,
    pub(crate) m_row_count: u32,
    pub(crate) m_row_size: u8,
    pub(crate) m_columns: [Column; 6],
    pub(crate) m_data: Option<&'db [u8]>,
}

impl<'db, T> TableInfo<'db, T> {
    pub(crate) fn set_columns<Tuple>(&mut self, tup: Tuple) where T: TableDesc<Columns=Tuple>, Tuple: ColumnTuple
    {
        assert!(self.m_row_size == 0);
        self.m_row_size = tup.row_size();
        tup.init(&mut self.m_columns);
        //println!("{:?}", self.m_columns);
    }

    pub(crate) fn set_row_count(&mut self, count: u32) {
        self.m_row_count = count;
    }

    pub(crate) fn index_size(&self) -> DynamicSize {
        if self.m_row_count < (1 << 16) { DynamicSize::Size2 } else { DynamicSize::Size4 }
    }

    pub(crate) fn set_data(&mut self, view: &'db [u8]) -> &'db [u8]
    {
        assert!(self.m_data.is_none());

        if self.m_row_count > 0 {
            assert!(self.m_row_size != 0);
            let (left, right) = view.split_at(self.m_row_count as usize * self.m_row_size as usize);
            self.m_data = Some(left);
            right
        } else {
            view
        }
    }
}

impl<'db, T> Default for TableInfo<'db, T> {
   fn default() -> Self {
        TableInfo {
            p: ::std::marker::PhantomData,
            m_row_count: 0,
            m_row_size: 0,
            m_columns: [Default::default(); 6],
            m_data: None,
        }
    }
}


#[allow(non_snake_case)]
#[derive(Default)]
pub(crate) struct Tables<'db> {
    TypeRef: TableInfo<'db, schema::TypeRef>,
    GenericParamConstraint: TableInfo<'db, schema::GenericParamConstraint>,
    TypeSpec: TableInfo<'db, schema::TypeSpec>,
    TypeDef: TableInfo<'db, schema::TypeDef>,
    CustomAttribute: TableInfo<'db, schema::CustomAttribute>,
    MethodDef: TableInfo<'db, schema::MethodDef>,
    MemberRef: TableInfo<'db, schema::MemberRef>,
    Module: TableInfo<'db, schema::Module>,
    Param: TableInfo<'db, schema::Param>,
    InterfaceImpl: TableInfo<'db, schema::InterfaceImpl>,
    Constant: TableInfo<'db, schema::Constant>,
    Field: TableInfo<'db, schema::Field>,
    FieldMarshal: TableInfo<'db, schema::FieldMarshal>,
    DeclSecurity: TableInfo<'db, schema::DeclSecurity>,
    ClassLayout: TableInfo<'db, schema::ClassLayout>,
    FieldLayout: TableInfo<'db, schema::FieldLayout>,
    StandAloneSig: TableInfo<'db, schema::StandAloneSig>,
    EventMap: TableInfo<'db, schema::EventMap>,
    Event: TableInfo<'db, schema::Event>,
    PropertyMap: TableInfo<'db, schema::PropertyMap>,
    Property: TableInfo<'db, schema::Property>,
    MethodSemantics: TableInfo<'db, schema::MethodSemantics>,
    MethodImpl: TableInfo<'db, schema::MethodImpl>,
    ModuleRef: TableInfo<'db, schema::ModuleRef>,
    ImplMap: TableInfo<'db, schema::ImplMap>,
    FieldRVA: TableInfo<'db, schema::FieldRVA>,
    Assembly: TableInfo<'db, schema::Assembly>,
    AssemblyProcessor: TableInfo<'db, schema::AssemblyProcessor>,
    AssemblyOS: TableInfo<'db, schema::AssemblyOS>,
    AssemblyRef: TableInfo<'db, schema::AssemblyRef>,
    AssemblyRefProcessor: TableInfo<'db, schema::AssemblyRefProcessor>,
    AssemblyRefOS: TableInfo<'db, schema::AssemblyRefOS>,
    File: TableInfo<'db, schema::File>,
    ExportedType: TableInfo<'db, schema::ExportedType>,
    ManifestResource: TableInfo<'db, schema::ManifestResource>,
    NestedClass: TableInfo<'db, schema::NestedClass>,
    GenericParam: TableInfo<'db, schema::GenericParam>,
    MethodSpec: TableInfo<'db, schema::MethodSpec>,
}

impl<'db> Tables<'db> {
    pub fn get_table_info<T: TableDesc>(&self) -> &TableInfo<'db, T> where Self: TableAccess<'db, T> {
        <Self as TableAccess<'db, T>>::get_table_info(self)
    }
}

pub(crate) trait CodedIndex : Sized {
    type Database;
    type Tables;

    fn decode(idx: u32, db: Self::Database) -> Result<Option<Self>>;
    fn index_size(tables: Self::Tables) -> DynamicSize;
    fn needs_4byte_index(row_count: u32, tag_bits: u8) -> bool {
        row_count >= (1u32 << (16 - tag_bits))
    }
}

pub struct Database<'db> {
    m_view: &'db [u8],
    m_strings: &'db [u8],
    m_blobs: &'db [u8],
    m_guids: &'db [u8],
    pub(crate) m_tables: Tables<'db>
}

pub fn is_database<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let file = File::open(path.as_ref())?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    if mmap.len() < mem::size_of::<pe::image_dos_header>() {
        return Ok(false);
    }
    
    let dos = unsafe { mmap.view_as::<pe::image_dos_header>(0) };

    if dos.e_magic != 0x5A4D { // IMAGE_DOS_SIGNATURE
        return Ok(false);
    }

    if mmap.len() < (dos.e_lfanew as usize + mem::size_of::<pe::image_nt_headers32>()) {
        return Ok(false);
    }

    let pe = unsafe { mmap.view_as::<pe::image_nt_headers32>(dos.e_lfanew as usize) };

    if pe.FileHeader.NumberOfSections == 0 || pe.FileHeader.NumberOfSections > 100 {
        return Ok(false);
    }

    let com = &pe.OptionalHeader.DataDirectory[14]; // IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR
    let sections = unsafe { mmap.view_as_slice::<pe::image_section_header>(dos.e_lfanew as usize + mem::size_of::<pe::image_nt_headers32>(),
                                                                            pe.FileHeader.NumberOfSections as usize) };
    let section = match pe::section_from_rva(sections, com.VirtualAddress) {
        None => return Ok(false),
        Some(s) => s
    };

    let offset = pe::offset_from_rva(section, com.VirtualAddress);

    let cli = unsafe { mmap.view_as::<pe::image_cor20_header>(offset) };

    if cli.cb as usize != mem::size_of::<pe::image_cor20_header>() {
        return Ok(false);
    }

    let section = match pe::section_from_rva(sections, cli.MetaData.VirtualAddress) {
        None => return Ok(false),
        Some(s) => s
    };

    let offset = pe::offset_from_rva(section, cli.MetaData.VirtualAddress);

    if *unsafe { mmap.view_as::<u32>(offset)} != 0x424a5342 {
        return Ok(false);
    }

    Ok(true)
}

#[repr(C)]
struct stream_range {
    offset: u32,
    size: u32,
}

fn stream_offset(name: &[u8]) -> usize {
    let mut padding = 4 - name.len() % 4;

    if padding == 0 {
        padding = 4;
    }

    (8 + name.len() + padding)
}

pub fn mmap_file<P: AsRef<Path>>(path: P) -> io::Result<Mmap> {
    let file = File::open(path.as_ref())?;
    unsafe { MmapOptions::new().map(&file) }
}

impl<'db> Database<'db> {
    pub fn load(data: &'db [u8]) -> Result<Database<'db>> {

        let m_view = data;

        if m_view.len() < mem::size_of::<pe::image_dos_header>() {
            return Err("Unexpected end of file".into());
        }
        
        let dos = unsafe { m_view.view_as::<pe::image_dos_header>(0) };

        if dos.e_magic != 0x5A4D { // IMAGE_DOS_SIGNATURE
            return Err("Invalid DOS signature".into());
        }

        if m_view.len() < (dos.e_lfanew as usize + mem::size_of::<pe::image_nt_headers32>()) {
            return Err("Unexpected end of file".into());
        }

        let pe = unsafe { m_view.view_as::<pe::image_nt_headers32>(dos.e_lfanew as usize) };

        if pe.FileHeader.NumberOfSections == 0 || pe.FileHeader.NumberOfSections > 100 {
            return Err("Invalid PE section count".into());
        }

        let com = &pe.OptionalHeader.DataDirectory[14]; // IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR
        let sections = unsafe { m_view.view_as_slice::<pe::image_section_header>(dos.e_lfanew as usize + mem::size_of::<pe::image_nt_headers32>(),
                                                                                pe.FileHeader.NumberOfSections as usize) };
        let section = match pe::section_from_rva(sections, com.VirtualAddress) {
            None => return Err("PE section containing CLI header not found".into()),
            Some(s) => s
        };

        let offset = pe::offset_from_rva(section, com.VirtualAddress);

        let cli = unsafe { m_view.view_as::<pe::image_cor20_header>(offset) };

        if cli.cb as usize != mem::size_of::<pe::image_cor20_header>() {
            return Err("Invalid CLI header".into());
        }

        let section = match pe::section_from_rva(sections, cli.MetaData.VirtualAddress) {
            None => return Err("PE section containing CLI metadata not found".into()),
            Some(s) => s
        };

        let offset = pe::offset_from_rva(section, cli.MetaData.VirtualAddress);

        if *unsafe { m_view.view_as::<u32>(offset)} != 0x424a5342 {
            return Err("CLI metadata magic signature not found".into());
        }

        let version_length = *unsafe { m_view.view_as::<u32>(offset + 12) } as usize;
        let stream_count = *unsafe {m_view.view_as::<u16>(offset + version_length + 18) };
        let mut view = &m_view[offset + version_length + 20..];
        let mut tables: Option<_> = None;

        let mut m_strings: Option<_> = None;
        let mut m_blobs: Option<_> = None;
        let mut m_guids: Option<_> = None;

        for _ in 0..stream_count {
            let stream = unsafe { view.view_as::<stream_range>(0) };
            let name = view.as_c_str(8);

            match name {
                b"#Strings" => {
                    m_strings = Some(m_view.sub(offset + stream.offset as usize, stream.size as usize))
                },
                b"#Blob" => {
                    m_blobs = Some(m_view.sub(offset + stream.offset as usize, stream.size as usize))
                },
                b"#GUID" => {
                    m_guids = Some(m_view.sub(offset + stream.offset as usize, stream.size as usize))
                },
                b"#~" => {
                    tables = Some(m_view.sub(offset + stream.offset as usize, stream.size as usize))
                },
                _ => {
                    if name != b"#US" {
                        return Err("Unknown metadata stream".into());
                    }
                }
            }

            view = &view[stream_offset(name)..];
        }

        let m_strings = match m_strings {
            Some(v) => v,
            None => return Err("Missing Strings stream".into())
        };
        let m_blobs = match m_blobs {
            Some(v) => v,
            None => return Err("Missing Blob stream".into())
        };
        let m_guids = match m_guids {
            Some(v) => v,
            None => return Err("Missing GUID stream".into())
        };
        let tables = match tables {
            Some(v) => v,
            None => return Err("Missing tables stream".into())
        };

        let heap_sizes = *unsafe { tables.view_as::<u8>(6) };
        let string_index_size = if heap_sizes >> 0 & 1 == 1 { DynamicSize::Size4 } else { DynamicSize::Size2 };
        let guid_index_size = if heap_sizes >> 1 & 1 == 1 { DynamicSize::Size4 } else { DynamicSize::Size2 };
        let blob_index_size = if heap_sizes >> 2 & 1 == 1 { DynamicSize::Size4 } else { DynamicSize::Size2 };

        let valid_bits = *unsafe { tables.view_as::<u64>(8) };
        let mut view = &tables[24..];

        let mut t = Tables::default();

        for i in 0..64
        {
            if valid_bits >> i & 1 == 0
            {
                continue;
            }

            let row_count = *unsafe {view.view_as::<u32>(0) };
            view = &view[4..];

            match i {
                0x00 => t.Module.set_row_count(row_count),
                0x01 => t.TypeRef.set_row_count(row_count),
                0x02 => t.TypeDef.set_row_count(row_count),
                0x04 => t.Field.set_row_count(row_count),
                0x06 => t.MethodDef.set_row_count(row_count),
                0x08 => t.Param.set_row_count(row_count),
                0x09 => t.InterfaceImpl.set_row_count(row_count),
                0x0a => t.MemberRef.set_row_count(row_count),
                0x0b => t.Constant.set_row_count(row_count),
                0x0c => t.CustomAttribute.set_row_count(row_count),
                0x0d => t.FieldMarshal.set_row_count(row_count),
                0x0e => t.DeclSecurity.set_row_count(row_count),
                0x0f => t.ClassLayout.set_row_count(row_count),
                0x10 => t.FieldLayout.set_row_count(row_count),
                0x11 => t.StandAloneSig.set_row_count(row_count),
                0x12 => t.EventMap.set_row_count(row_count),
                0x14 => t.Event.set_row_count(row_count),
                0x15 => t.PropertyMap.set_row_count(row_count),
                0x17 => t.Property.set_row_count(row_count),
                0x18 => t.MethodSemantics.set_row_count(row_count),
                0x19 => t.MethodImpl.set_row_count(row_count),
                0x1a => t.ModuleRef.set_row_count(row_count),
                0x1b => t.TypeSpec.set_row_count(row_count),
                0x1c => t.ImplMap.set_row_count(row_count),
                0x1d => t.FieldRVA.set_row_count(row_count),
                0x20 => t.Assembly.set_row_count(row_count),
                0x21 => t.AssemblyProcessor.set_row_count(row_count),
                0x22 => t.AssemblyOS.set_row_count(row_count),
                0x23 => t.AssemblyRef.set_row_count(row_count),
                0x24 => t.AssemblyRefProcessor.set_row_count(row_count),
                0x25 => t.AssemblyRefOS.set_row_count(row_count),
                0x26 => t.File.set_row_count(row_count),
                0x27 => t.ExportedType.set_row_count(row_count),
                0x28 => t.ManifestResource.set_row_count(row_count),
                0x29 => t.NestedClass.set_row_count(row_count),
                0x2a => t.GenericParam.set_row_count(row_count),
                0x2b => t.MethodSpec.set_row_count(row_count),
                0x2c => t.GenericParamConstraint.set_row_count(row_count),
                _ => return Err("Unknown metadata table".into())
            }
        }

        let type_def_or_ref_index_size = schema::TypeDefOrRef::index_size(&t);
        let method_def_or_ref_index_size = schema::MethodDefOrRef::index_size(&t);
        let implementation_index_size = schema::Implementation::index_size(&t);

        t.Assembly.set_columns((FixedSize4, FixedSize8, FixedSize4, blob_index_size, string_index_size, string_index_size));
        t.AssemblyOS.set_columns((FixedSize4, FixedSize4, FixedSize4));
        t.AssemblyProcessor.set_columns((FixedSize4,));
        t.AssemblyRef.set_columns((FixedSize8, FixedSize4, blob_index_size, string_index_size, string_index_size, blob_index_size));
        t.AssemblyRefOS.set_columns((FixedSize4, FixedSize4, FixedSize4, t.AssemblyRef.index_size()));
        t.AssemblyRefProcessor.set_columns((FixedSize4, t.AssemblyRef.index_size()));
        t.ClassLayout.set_columns((FixedSize2, FixedSize4, t.TypeDef.index_size()));
        t.Constant.set_columns((FixedSize2, schema::HasConstant::index_size(&t), blob_index_size));
        t.CustomAttribute.set_columns((schema::HasCustomAttribute::index_size(&t), schema::CustomAttributeType::index_size(&t), blob_index_size));
        t.DeclSecurity.set_columns((FixedSize2, schema::HasDeclSecurity::index_size(&t), blob_index_size));
        t.EventMap.set_columns((t.TypeDef.index_size(), t.Event.index_size()));
        t.Event.set_columns((FixedSize2, string_index_size, type_def_or_ref_index_size));
        t.ExportedType.set_columns((FixedSize4, FixedSize4, string_index_size, string_index_size, implementation_index_size));
        t.Field.set_columns((FixedSize2, string_index_size, blob_index_size));
        t.FieldLayout.set_columns((FixedSize4, t.Field.index_size()));
        t.FieldMarshal.set_columns((schema::HasFieldMarshal::index_size(&t), blob_index_size));
        t.FieldRVA.set_columns((FixedSize4, t.Field.index_size()));
        t.File.set_columns((FixedSize4, string_index_size, blob_index_size));
        t.GenericParam.set_columns((FixedSize2, FixedSize2, schema::TypeOrMethodDef::index_size(&t), string_index_size));
        t.GenericParamConstraint.set_columns((t.GenericParam.index_size(), type_def_or_ref_index_size));
        t.ImplMap.set_columns((FixedSize2, schema::MemberForwarded::index_size(&t), string_index_size, t.ModuleRef.index_size()));
        t.InterfaceImpl.set_columns((t.TypeDef.index_size(), type_def_or_ref_index_size));
        t.ManifestResource.set_columns((FixedSize4, FixedSize4, string_index_size, implementation_index_size));
        t.MemberRef.set_columns((schema::MemberRefParent::index_size(&t), string_index_size, blob_index_size));
        t.MethodDef.set_columns((FixedSize4, FixedSize2, FixedSize2, string_index_size, blob_index_size, t.Param.index_size()));
        t.MethodImpl.set_columns((t.TypeDef.index_size(), method_def_or_ref_index_size, method_def_or_ref_index_size));
        t.MethodSemantics.set_columns((FixedSize2, t.MethodDef.index_size(), schema::HasSemantics::index_size(&t)));
        t.MethodSpec.set_columns((method_def_or_ref_index_size, blob_index_size));
        t.Module.set_columns((FixedSize2, string_index_size, guid_index_size, guid_index_size, guid_index_size));
        t.ModuleRef.set_columns((string_index_size,));
        t.NestedClass.set_columns((t.TypeDef.index_size(), t.TypeDef.index_size()));
        t.Param.set_columns((FixedSize2, FixedSize2, string_index_size));
        t.Property.set_columns((FixedSize2, string_index_size, blob_index_size));
        t.PropertyMap.set_columns((t.TypeDef.index_size(), t.Property.index_size()));
        t.StandAloneSig.set_columns((blob_index_size,));
        t.TypeDef.set_columns((FixedSize4, string_index_size, string_index_size, type_def_or_ref_index_size, t.Field.index_size(), t.MethodDef.index_size()));
        t.TypeRef.set_columns((schema::ResolutionScope::index_size(&t), string_index_size, string_index_size));
        t.TypeSpec.set_columns((blob_index_size,));

        view = t.Module.set_data(view);
        view = t.TypeRef.set_data(view);
        view = t.TypeDef.set_data(view);
        view = t.Field.set_data(view);
        view = t.MethodDef.set_data(view);
        view = t.Param.set_data(view);
        view = t.InterfaceImpl.set_data(view);
        view = t.MemberRef.set_data(view);
        view = t.Constant.set_data(view);
        view = t.CustomAttribute.set_data(view);
        view = t.FieldMarshal.set_data(view);
        view = t.DeclSecurity.set_data(view);
        view = t.ClassLayout.set_data(view);
        view = t.FieldLayout.set_data(view);
        view = t.StandAloneSig.set_data(view);
        view = t.EventMap.set_data(view);
        view = t.Event.set_data(view);
        view = t.PropertyMap.set_data(view);
        view = t.Property.set_data(view);
        view = t.MethodSemantics.set_data(view);
        view = t.MethodImpl.set_data(view);
        view = t.ModuleRef.set_data(view);
        view = t.TypeSpec.set_data(view);
        view = t.ImplMap.set_data(view);
        view = t.FieldRVA.set_data(view);
        view = t.Assembly.set_data(view);
        view = t.AssemblyProcessor.set_data(view);
        view = t.AssemblyOS.set_data(view);
        view = t.AssemblyRef.set_data(view);
        view = t.AssemblyRefProcessor.set_data(view);
        view = t.AssemblyRefOS.set_data(view);
        view = t.File.set_data(view);
        view = t.ExportedType.set_data(view);
        view = t.ManifestResource.set_data(view);
        view = t.NestedClass.set_data(view);
        view = t.GenericParam.set_data(view);
        view = t.MethodSpec.set_data(view);
        t.GenericParamConstraint.set_data(view);

        Ok(Database {
            m_view: data,
            m_strings,
            m_blobs,
            m_guids,
            m_tables: t
        })
    }

    pub(crate) fn get_table_info<T: TableKind>(&self) -> &TableInfo<'db, T> where Self: TableAccess<'db, T> {
        <Self as TableAccess<'db, T>>::get_table_info(self)
    }

    pub fn get_table<T: TableKind>(&'db self) -> Table<'db, T>
        where Self: TableAccess<'db, T>
    {
        Table {
            db: self,
            table: self.get_table_info::<T>()
        }
    }

    pub(crate) fn get_string(&self, index: u32) -> Result<&str> {
        let view = &self.m_strings[index as usize..];
        let len = match view.iter().position(|b| *b == b'\0') {
            Some(p) => p,
            None => return Err("Missing string terminator".into())
        };

        Ok(std::str::from_utf8(&view[..len])?)
    }

    pub(crate) fn get_blob(&self, index: u32) -> Result<&[u8]> {
        let view = &self.m_blobs[index as usize..];
        let mut initial_byte: u8 = view[0];
        let blob_size_bytes: usize = match initial_byte >> 5 {
            0 | 1 | 2 | 3 => {
                initial_byte &= 0x7f;
                1
            },
            4 | 5 => {
                initial_byte &= 0x3f;
                2
            },
            6 => {
                initial_byte &= 0x1f;
                4
            },
            _ => return Err("Invalid blob encoding".into())
        };

        let mut blob_size = initial_byte as usize;

        for &byte in view.sub(1, blob_size_bytes - 1)
        {
            blob_size = (blob_size << 8) + byte as usize;
        }

        Ok(view.sub(blob_size_bytes, blob_size))
    }
}
