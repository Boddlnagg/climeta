use std::fs::File;
use std::path::Path;
use std::mem;
use std::io;

use crate::Result;
use crate::schema;
use crate::TableRow;
use crate::core::pe;
use crate::core::table::{TableInfo, Table};
use crate::core::ByteView;
use crate::core::columns::{ColumnIndex, FixedSize2, FixedSize4, FixedSize8, DynamicSize};


pub trait TableKind: Copy {}

pub(crate) trait TableDesc: TableKind {
    type Columns;
}

pub(crate) trait TableDescWithKey: TableDesc /*where Self: ColumnAccess<Self::KeyColumn>*/ {
    type KeyColumn: ColumnIndex;
}

pub(crate) trait TableInfoAccess<'db, T> {
    fn get_table_info(&self) -> &TableInfo<'db, T>;
}

macro_rules! impl_tableinfo_access {
    ( $tab:ident ) => {
        impl<'db> TableInfoAccess<'db, schema::marker::$tab> for Tables<'db> {
            fn get_table_info(&self) -> &TableInfo<'db, schema::marker::$tab> {
                &self.$tab
            }
        }

        impl<'db> TableInfoAccess<'db, schema::marker::$tab> for Database<'db> {
            fn get_table_info(&self) -> &TableInfo<'db, schema::marker::$tab> {
                &self.tables.$tab
            }
        }
    }
}

impl_tableinfo_access!(TypeRef);
impl_tableinfo_access!(GenericParamConstraint);
impl_tableinfo_access!(TypeSpec);
impl_tableinfo_access!(TypeDef);
impl_tableinfo_access!(CustomAttribute);
impl_tableinfo_access!(MethodDef);
impl_tableinfo_access!(MemberRef);
impl_tableinfo_access!(Module);
impl_tableinfo_access!(Param);
impl_tableinfo_access!(InterfaceImpl);
impl_tableinfo_access!(Constant);
impl_tableinfo_access!(Field);
impl_tableinfo_access!(FieldMarshal);
impl_tableinfo_access!(DeclSecurity);
impl_tableinfo_access!(ClassLayout);
impl_tableinfo_access!(FieldLayout);
impl_tableinfo_access!(StandAloneSig);
impl_tableinfo_access!(EventMap);
impl_tableinfo_access!(Event);
impl_tableinfo_access!(PropertyMap);
impl_tableinfo_access!(Property);
impl_tableinfo_access!(MethodSemantics);
impl_tableinfo_access!(MethodImpl);
impl_tableinfo_access!(ModuleRef);
impl_tableinfo_access!(ImplMap);
impl_tableinfo_access!(FieldRVA);
impl_tableinfo_access!(Assembly);
impl_tableinfo_access!(AssemblyProcessor);
impl_tableinfo_access!(AssemblyOS);
impl_tableinfo_access!(AssemblyRef);
impl_tableinfo_access!(AssemblyRefProcessor);
impl_tableinfo_access!(AssemblyRefOS);
impl_tableinfo_access!(File);
impl_tableinfo_access!(ExportedType);
impl_tableinfo_access!(ManifestResource);
impl_tableinfo_access!(NestedClass);
impl_tableinfo_access!(GenericParam);
impl_tableinfo_access!(MethodSpec);


#[allow(non_snake_case)]
#[derive(Default)]
pub(crate) struct Tables<'db> {
    TypeRef: TableInfo<'db, schema::marker::TypeRef>,
    GenericParamConstraint: TableInfo<'db, schema::marker::GenericParamConstraint>,
    TypeSpec: TableInfo<'db, schema::marker::TypeSpec>,
    TypeDef: TableInfo<'db, schema::marker::TypeDef>,
    CustomAttribute: TableInfo<'db, schema::marker::CustomAttribute>,
    MethodDef: TableInfo<'db, schema::marker::MethodDef>,
    MemberRef: TableInfo<'db, schema::marker::MemberRef>,
    Module: TableInfo<'db, schema::marker::Module>,
    Param: TableInfo<'db, schema::marker::Param>,
    InterfaceImpl: TableInfo<'db, schema::marker::InterfaceImpl>,
    Constant: TableInfo<'db, schema::marker::Constant>,
    Field: TableInfo<'db, schema::marker::Field>,
    FieldMarshal: TableInfo<'db, schema::marker::FieldMarshal>,
    DeclSecurity: TableInfo<'db, schema::marker::DeclSecurity>,
    ClassLayout: TableInfo<'db, schema::marker::ClassLayout>,
    FieldLayout: TableInfo<'db, schema::marker::FieldLayout>,
    StandAloneSig: TableInfo<'db, schema::marker::StandAloneSig>,
    EventMap: TableInfo<'db, schema::marker::EventMap>,
    Event: TableInfo<'db, schema::marker::Event>,
    PropertyMap: TableInfo<'db, schema::marker::PropertyMap>,
    Property: TableInfo<'db, schema::marker::Property>,
    MethodSemantics: TableInfo<'db, schema::marker::MethodSemantics>,
    MethodImpl: TableInfo<'db, schema::marker::MethodImpl>,
    ModuleRef: TableInfo<'db, schema::marker::ModuleRef>,
    ImplMap: TableInfo<'db, schema::marker::ImplMap>,
    FieldRVA: TableInfo<'db, schema::marker::FieldRVA>,
    Assembly: TableInfo<'db, schema::marker::Assembly>,
    AssemblyProcessor: TableInfo<'db, schema::marker::AssemblyProcessor>,
    AssemblyOS: TableInfo<'db, schema::marker::AssemblyOS>,
    AssemblyRef: TableInfo<'db, schema::marker::AssemblyRef>,
    AssemblyRefProcessor: TableInfo<'db, schema::marker::AssemblyRefProcessor>,
    AssemblyRefOS: TableInfo<'db, schema::marker::AssemblyRefOS>,
    File: TableInfo<'db, schema::marker::File>,
    ExportedType: TableInfo<'db, schema::marker::ExportedType>,
    ManifestResource: TableInfo<'db, schema::marker::ManifestResource>,
    NestedClass: TableInfo<'db, schema::marker::NestedClass>,
    GenericParam: TableInfo<'db, schema::marker::GenericParam>,
    MethodSpec: TableInfo<'db, schema::marker::MethodSpec>,
}

impl<'db> Tables<'db> {
    pub fn get_table_info<T: TableDesc>(&self) -> &TableInfo<'db, T> where Self: TableInfoAccess<'db, T> {
        <Self as TableInfoAccess<'db, T>>::get_table_info(self)
    }
}

pub(crate) trait CodedIndex: Sized {
    type Database;
    type Tables;
    const TAG_BITS: u8;

    fn decode(idx: u32, db: Self::Database) -> Result<Option<Self>>;
    fn index_size(tables: Self::Tables) -> DynamicSize;
    fn needs_4byte_index(row_count: u32, tag_bits: u8) -> bool {
        row_count >= (1u32 << (16 - tag_bits))
    }
    fn encode<T: TableRow>(row: &T) -> u32
        where Self: CodedIndexEncode<T::Kind>
    {
        ((row.get_index() + 1) << Self::TAG_BITS as u32) | <Self as CodedIndexEncode<T::Kind>>::TAG as u32
    }
}

pub(crate) trait CodedIndexEncode<T: TableKind>: CodedIndex {
    const TAG: u8;
}

pub(crate) struct Database<'db> {
    strings: &'db [u8],
    blobs: &'db [u8],
    _guids: &'db [u8],
    tables: Tables<'db>,
    asm_name: Option<&'db str>,
}

pub fn is_database<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let file = File::open(path.as_ref())?;
    let mmap = unsafe { memmap::Mmap::map(&file)? };

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

    if *unsafe { mmap.view_as::<u32>(offset) } != 0x424a5342 {
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

fn read_string(strings: &[u8], index: u32) -> Result<&str> {
        let view = &strings[index as usize..];
        let len = match view.iter().position(|b| *b == b'\0') {
            Some(p) => p,
            None => return Err("Missing string terminator".into())
        };

        std::str::from_utf8(&view[..len]).map_err(|_| "Invalid UTF8 in string heap".into())
    }

impl<'db> Database<'db> {
    pub fn load(data: &'db [u8]) -> Result<Database<'db>> {

        let view = data;

        if view.len() < mem::size_of::<pe::image_dos_header>() {
            return Err("Unexpected end of file".into());
        }
        
        let dos = unsafe { view.view_as::<pe::image_dos_header>(0) };

        if dos.e_magic != 0x5A4D { // IMAGE_DOS_SIGNATURE
            return Err("Invalid DOS signature".into());
        }

        if view.len() < (dos.e_lfanew as usize + mem::size_of::<pe::image_nt_headers32>()) {
            return Err("Unexpected end of file".into());
        }

        let pe = unsafe { view.view_as::<pe::image_nt_headers32>(dos.e_lfanew as usize) };

        if pe.FileHeader.NumberOfSections == 0 || pe.FileHeader.NumberOfSections > 100 {
            return Err("Invalid PE section count".into());
        }

        let com = &pe.OptionalHeader.DataDirectory[14]; // IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR
        let sections = unsafe { view.view_as_slice::<pe::image_section_header>(dos.e_lfanew as usize + mem::size_of::<pe::image_nt_headers32>(),
                                                                               pe.FileHeader.NumberOfSections as usize) };
        let section = match pe::section_from_rva(sections, com.VirtualAddress) {
            None => return Err("PE section containing CLI header not found".into()),
            Some(s) => s
        };

        let offset = pe::offset_from_rva(section, com.VirtualAddress);

        let cli = unsafe { view.view_as::<pe::image_cor20_header>(offset) };

        if cli.cb as usize != mem::size_of::<pe::image_cor20_header>() {
            return Err("Invalid CLI header".into());
        }

        let section = match pe::section_from_rva(sections, cli.MetaData.VirtualAddress) {
            None => return Err("PE section containing CLI metadata not found".into()),
            Some(s) => s
        };

        let offset = pe::offset_from_rva(section, cli.MetaData.VirtualAddress);

        if *unsafe { view.view_as::<u32>(offset) } != 0x424a5342 {
            return Err("CLI metadata magic signature not found".into());
        }

        let version_length = *unsafe { view.view_as::<u32>(offset + 12) } as usize;
        let stream_count = *unsafe { view.view_as::<u16>(offset + version_length + 18) };
        let mut remaining = &view[offset + version_length + 20..];
        let mut tables: Option<_> = None;

        let mut strings: Option<_> = None;
        let mut blobs: Option<_> = None;
        let mut guids: Option<_> = None;

        for _ in 0..stream_count {
            let stream = unsafe { remaining.view_as::<stream_range>(0) };
            let name = remaining.as_c_str(8);

            match name {
                b"#Strings" => {
                    strings = Some(view.sub(offset + stream.offset as usize, stream.size as usize))
                },
                b"#Blob" => {
                    blobs = Some(view.sub(offset + stream.offset as usize, stream.size as usize))
                },
                b"#GUID" => {
                    guids = Some(view.sub(offset + stream.offset as usize, stream.size as usize))
                },
                b"#~" => {
                    tables = Some(view.sub(offset + stream.offset as usize, stream.size as usize))
                },
                _ => {
                    if name != b"#US" {
                        return Err("Unknown metadata stream".into());
                    }
                }
            }

            remaining = &remaining[stream_offset(name)..];
        }

        let strings = match strings {
            Some(v) => v,
            None => return Err("Missing Strings stream".into())
        };
        let blobs = match blobs {
            Some(v) => v,
            None => return Err("Missing Blob stream".into())
        };
        let guids = match guids {
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

        for i in 0..64 {
            if valid_bits >> i & 1 == 0 {
                continue;
            }

            let row_count = *unsafe { view.view_as::<u32>(0) };
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

        let asm_name =
            if t.Assembly.len() > 0 {
                t.Assembly.get_value::<crate::core::columns::Col4, u32>(0).and_then(|idx| read_string(strings, idx)).ok()
            } else {
                None
            };

        Ok(Database {
            strings: strings,
            blobs: blobs,
            _guids: guids,
            tables: t,
            asm_name: asm_name
        })
    }

    pub(crate) fn get_table_info<T: TableKind>(&self) -> &TableInfo<'db, T> where Self: TableInfoAccess<'db, T> {
        <Self as TableInfoAccess<'db, T>>::get_table_info(self)
    }

    pub fn get_table<T: crate::TableRow>(&'db self) -> Table<'db, T::Kind>
        where Self: TableInfoAccess<'db, T::Kind>
    {
        Table {
            db: self,
            table: self.get_table_info::<T::Kind>()
        }
    }

    pub(crate) fn get_string(&self, index: u32) -> Result<&str> {
        read_string(self.strings, index)
    }

    pub(crate) fn get_blob(&self, index: u32) -> Result<&[u8]> {
        let view = &self.blobs[index as usize..];
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

        for &byte in view.sub(1, blob_size_bytes - 1) {
            blob_size = (blob_size << 8) + byte as usize;
        }

        Ok(view.sub(blob_size_bytes, blob_size))
    }
}

impl<'db> crate::AssemblyAccess for Database<'db> {
    fn get_assembly(&self) -> Option<schema::Assembly> {
        let table = self.get_table::<schema::Assembly>();
        if table.len() > 0 {
            match table.get_row(0) {
                Ok(row) => Some(row),
                Err(_) => None
            }
        } else {
            None
        }
    }

    fn assembly_name(&self) -> Option<&str> {
        self.asm_name
    }
}
