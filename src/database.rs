use std::fs::File;
use std::path::Path;
use std::mem;
use std::io;

use byteorder::{ByteOrder, LittleEndian};
use memmap::{MmapOptions, Mmap};

use crate::{pe, schema};
use crate::table::{Table, Column};
use crate::Result;
use crate::ByteView;

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

pub trait ColumnSize: Copy {
    fn size(&self) -> u8;
}

#[derive(Copy, Clone, Default)]
pub struct FixedSize2;
impl ColumnSize for FixedSize2 {
    fn size(&self) -> u8 { 2 }
}

#[derive(Copy, Clone, Default)]
pub struct FixedSize4;
impl ColumnSize for FixedSize4 {
    fn size(&self) -> u8 { 4 }
}

#[derive(Copy, Clone, Default)]
pub struct FixedSize8;
impl ColumnSize for FixedSize8 {
    fn size(&self) -> u8 { 8 }
}

#[derive(Copy, Clone, Debug)]
pub enum DynamicSize {
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

pub trait TableDesc {
    type Columns;
}

pub trait TableAccess<'a, T: TableDesc> {
    fn get_table(&self) -> &Table<'a, T>;
}

macro_rules! impl_table_access {
    ( $tab:ident ) => {
        impl<'a> TableAccess<'a, schema::$tab> for Tables<'a> {
            fn get_table(&self) -> &Table<'a, schema::$tab> {
                &self.$tab
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

#[allow(non_snake_case)]
#[derive(Default)]
pub struct Tables<'a> {
    TypeRef: Table<'a, schema::TypeRef>,
    GenericParamConstraint: Table<'a, schema::GenericParamConstraint>,
    TypeSpec: Table<'a, schema::TypeSpec>,
    TypeDef: Table<'a, schema::TypeDef>,
    CustomAttribute: Table<'a, schema::CustomAttribute>,
    MethodDef: Table<'a, schema::MethodDef>,
    MemberRef: Table<'a, schema::MemberRef>,
    Module: Table<'a, schema::Module>,
    Param: Table<'a, schema::Param>,
    InterfaceImpl: Table<'a, schema::InterfaceImpl>,
    Constant: Table<'a, schema::Constant>,
    Field: Table<'a, schema::Field>,
    FieldMarshal: Table<'a, schema::FieldMarshal>,
    DeclSecurity: Table<'a, schema::DeclSecurity>,
    ClassLayout: Table<'a, schema::ClassLayout>,
    FieldLayout: Table<'a, schema::FieldLayout>,
    StandAloneSig: Table<'a, schema::StandAloneSig>,
    EventMap: Table<'a, schema::EventMap>,
    Event: Table<'a, schema::Event>,
    PropertyMap: Table<'a, schema::PropertyMap>,
    Property: Table<'a, schema::Property>,
    MethodSemantics: Table<'a, schema::MethodSemantics>,
    MethodImpl: Table<'a, schema::MethodImpl>,
    ModuleRef: Table<'a, schema::ModuleRef>,
    ImplMap: Table<'a, schema::ImplMap>,
    FieldRVA: Table<'a, schema::FieldRVA>,
    Assembly: Table<'a, schema::Assembly>,
    AssemblyProcessor: Table<'a, schema::AssemblyProcessor>,
    AssemblyOS: Table<'a, schema::AssemblyOS>,
    AssemblyRef: Table<'a, schema::AssemblyRef>,
    AssemblyRefProcessor: Table<'a, schema::AssemblyRefProcessor>,
    AssemblyRefOS: Table<'a, schema::AssemblyRefOS>,
    File: Table<'a, schema::File>,
    ExportedType: Table<'a, schema::ExportedType>,
    ManifestResource: Table<'a, schema::ManifestResource>,
    NestedClass: Table<'a, schema::NestedClass>,
    GenericParam: Table<'a, schema::GenericParam>,
    MethodSpec: Table<'a, schema::MethodSpec>,
}

impl<'a> Tables<'a> {
    pub fn get_table<T: TableDesc>(&self) -> &Table<'a, T> where Self: TableAccess<'a, T> {
        <Self as TableAccess<'a, T>>::get_table(self)
    }
}

pub(crate) trait CodedIndex : Sized {
    type Database;
    type Tables;

    fn decode(idx: u32, tables: Self::Database) -> Result<Option<Self>>;
    fn index_size(tables: Self::Tables) -> DynamicSize;
    fn needs_4byte_index(row_count: u32, tag_bits: u8) -> bool {
        row_count >= (1u32 << (16 - tag_bits))
    }
}

pub struct Database<'a> {
    m_view: &'a [u8],
    m_strings: &'a [u8],
    m_blobs: &'a [u8],
    m_guids: &'a [u8],
    pub(crate) m_tables: Tables<'a>
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

impl<'a> Database<'a> {
    pub fn load(data: &'a [u8]) -> Result<Database<'a>> {

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

    pub fn get_table<T: TableDesc>(&self) -> &Table<'a, T>
        where Tables<'a>: TableAccess<'a, T>
    {
        self.m_tables.get_table::<T>()
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