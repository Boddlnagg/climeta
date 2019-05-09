use std::fmt;

use crate::{Result, Cache, ResolveToTypeDef};

use crate::core::db::{Database, Tables, CodedIndex, CodedIndexEncode};
use crate::core::columns::DynamicSize;

pub mod flags;
mod rows;
pub use rows::*;
mod signatures;
pub use signatures::*;
mod custom_attributes;
pub use custom_attributes::*;

macro_rules! table_kind {
    ($ty:ident [$($colty:ty),+]) => {
        #[derive(Copy, Clone)]
        pub struct $ty;

        impl TableKind for $ty {
            // unfortunately no generic associated type Row<'db> yet ...
        }

        impl<'a> crate::TableRowAccess for &'a $ty {
            type Table = Table<'a, $ty>;
            type Out = super::rows::$ty<'a>;

            fn get(table: Self::Table, row: u32) -> Self::Out {
                super::rows::$ty(Row::new(table, row))
            }
        }

        impl TableDesc for $ty {
            type Columns = ($($colty),+ ,);
        }
    };
    ($ty:ident [$($colty:ty),+] key $key:ident) => {
        table_kind!($ty [$($colty),+]);

        impl TableDescWithKey for $ty {
            type KeyColumn = crate::core::columns::$key;
        }
    }
}

pub mod marker {
    use crate::core::table::{Table, Row};
    use crate::core::db::{TableKind, TableDesc, TableDescWithKey};
    use crate::core::columns::{FixedSize2, FixedSize4, FixedSize8, DynamicSize};

    table_kind!(Assembly [FixedSize4, FixedSize8, FixedSize4, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(AssemblyOS [FixedSize4, FixedSize4, FixedSize4]);
    table_kind!(AssemblyProcessor [FixedSize4]);
    table_kind!(AssemblyRef [FixedSize8, FixedSize4, DynamicSize, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(AssemblyRefOS [FixedSize4, FixedSize4, FixedSize4, DynamicSize]);
    table_kind!(AssemblyRefProcessor [FixedSize4, DynamicSize]);
    table_kind!(ClassLayout [FixedSize2, FixedSize4, DynamicSize] key Col2 /*Parent*/);
    table_kind!(Constant [FixedSize2, DynamicSize, DynamicSize] key Col1 /*Parent*/);
    table_kind!(CustomAttribute [DynamicSize, DynamicSize, DynamicSize] key Col0 /*Parent*/);
    table_kind!(DeclSecurity [FixedSize2, DynamicSize, DynamicSize] key Col1 /*Parent*/);
    table_kind!(Event [FixedSize2, DynamicSize, DynamicSize]);
    table_kind!(EventMap [DynamicSize, DynamicSize]);
    table_kind!(ExportedType [FixedSize4, FixedSize4, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(Field [FixedSize2, DynamicSize, DynamicSize]);
    table_kind!(FieldLayout [FixedSize4, DynamicSize] key Col1 /*Field*/);
    table_kind!(FieldMarshal [DynamicSize, DynamicSize] key Col0 /*Parent*/);
    table_kind!(FieldRVA [FixedSize4, DynamicSize] key Col1 /*Field*/);
    table_kind!(File [FixedSize4, DynamicSize, DynamicSize]);
    table_kind!(GenericParam [FixedSize2, FixedSize2, DynamicSize, DynamicSize] key Col2 /*Owner*/);
    table_kind!(GenericParamConstraint [DynamicSize, DynamicSize] key Col0 /*Owner*/);
    table_kind!(ImplMap [FixedSize2, DynamicSize, DynamicSize, DynamicSize] key Col1 /*MemberForwarded*/);
    table_kind!(InterfaceImpl [DynamicSize, DynamicSize] key Col0 /*Class*/);
    table_kind!(ManifestResource [FixedSize4, FixedSize4, DynamicSize, DynamicSize]);
    table_kind!(MemberRef [DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(MethodDef [FixedSize4, FixedSize2, FixedSize2, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(MethodImpl [DynamicSize, DynamicSize, DynamicSize] key Col0 /*Class*/);
    table_kind!(MethodSemantics [FixedSize2, DynamicSize, DynamicSize] key Col2 /*Association*/);
    table_kind!(MethodSpec [DynamicSize, DynamicSize]);
    table_kind!(Module [FixedSize2, DynamicSize, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(ModuleRef [DynamicSize]);
    table_kind!(NestedClass [DynamicSize, DynamicSize] key Col0 /*NestedClass*/);
    table_kind!(Param [FixedSize2, FixedSize2, DynamicSize]);
    table_kind!(Property [FixedSize2, DynamicSize, DynamicSize]);
    table_kind!(PropertyMap [DynamicSize, DynamicSize]);
    table_kind!(StandAloneSig [DynamicSize]);
    table_kind!(TypeDef [FixedSize4, DynamicSize, DynamicSize, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(TypeRef [DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(TypeSpec [DynamicSize]);
}

macro_rules! coded_index {
    ($name:ident[$bits:tt] { $($n:tt => $ty:ident),+ }) => {

        #[derive(Clone)]
        pub enum $name<'db> {
            $($ty(rows::$ty<'db>)),+
        }

        impl<'db> CodedIndex for $name<'db> {
            type Database = &'db Database<'db>;
            type Tables = &'db Tables<'db>;
            const TAG_BITS: u8 = $bits;

            fn decode(idx: u32, db: Self::Database) -> Result<Option<Self>> {
                let tag = idx & ((1 << Self::TAG_BITS) - 1);
                let row = idx >> Self::TAG_BITS as u32;
                if row == 0 {
                    return Ok(None);
                }
                let row = row - 1;
                Ok(Some(match tag {
                    $($n => $name::$ty(db.get_table::<$ty>().get_row(row)?),)+
                    _ => unreachable!()
                }))
            }

            fn index_size(tables: Self::Tables) -> DynamicSize {
                if $(Self::needs_4byte_index(tables.get_table_info::<marker::$ty>().m_row_count, Self::TAG_BITS))||+ {
                    DynamicSize::Size4
                } else {
                    DynamicSize::Size2
                }
            }
        }

        $(
        impl<'db> CodedIndexEncode<marker::$ty> for $name<'db> {
            const TAG: u8 = $n;
        }
        )+

        impl<'db> std::fmt::Debug for $name<'db> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $($name::$ty(_) => write!(f, "{}::{}", stringify!($name), stringify!($ty))),+
                }
            }
        }
    }
}

coded_index! {
    TypeDefOrRef[2] {
        0 => TypeDef,
        1 => TypeRef,
        2 => TypeSpec
    }
}

coded_index! {
    HasConstant[2] {
        0 => Field,
        1 => Param,
        2 => Property
    }
}

coded_index! {
    HasCustomAttribute[5] {
        0 => MethodDef,
        1 => Field,
        2 => TypeRef,
        3 => TypeDef,
        4 => Param,
        5 => InterfaceImpl,
        6 => MemberRef,
        7 => Module,
        //8 => Permission,
        9 => Property,
        10 => Event,
        11 => StandAloneSig,
        12 => ModuleRef,
        13 => TypeSpec,
        14 => Assembly,
        15 => AssemblyRef,
        16 => File,
        17 => ExportedType,
        18 => ManifestResource,
        19 => GenericParam,
        20 => GenericParamConstraint,
        21 => MethodSpec
    }
}

coded_index! {
    HasFieldMarshal[1] {
        0 => Field,
        1 => Param
    }
}

coded_index! {
    HasDeclSecurity[2] {
        0 => TypeDef,
        1 => MethodDef,
        2 => Assembly
    }
}

coded_index! {
    MemberRefParent[3] {
        0 => TypeDef,
        1 => TypeRef,
        2 => ModuleRef,
        3 => MethodDef,
        4 => TypeSpec
    }
}

coded_index! {
    HasSemantics[1] {
        0 => Event,
        1 => Property
    }
}

coded_index! {
    MethodDefOrRef[1] {
        0 => MethodDef,
        1 => MemberRef
    }
}

coded_index! {
    MemberForwarded[1] {
        0 => Field,
        1 => MethodDef
    }
}

coded_index! {
    Implementation[2] {
        0 => File,
        1 => AssemblyRef,
        2 => ExportedType
    }
}

coded_index! {
    CustomAttributeType[3] {
        //0 => Not used
        //1 => Not used
        2 => MethodDef,
        3 => MemberRef
        //4 => Not used
    }
}

coded_index! {
    ResolutionScope[2] {
        0 => Module,
        1 => ModuleRef,
        2 => AssemblyRef,
        3 => TypeRef
    }
}

coded_index! {
    TypeOrMethodDef[1] {
        0 => TypeDef,
        1 => MethodDef
    }
}

impl<'db> ResolveToTypeDef<'db> for TypeDefOrRef<'db> {
    fn namespace_name_pair(&self) -> (&'db str, &'db str) {
        match self {
            TypeDefOrRef::TypeDef(d) => d.namespace_name_pair(),
            TypeDefOrRef::TypeRef(r) => r.namespace_name_pair(),
            TypeDefOrRef::TypeSpec(_s) => panic!("TypeSpec has no namespace/name pair"),
        }
    }

    fn resolve<'c: 'db>(&self, cache: &Cache<'c>) -> Option<TypeDef<'db>> {
        match self {
            TypeDefOrRef::TypeDef(d) => Some(d.clone()),
            TypeDefOrRef::TypeRef(r) => r.resolve(cache),
            TypeDefOrRef::TypeSpec(_s) => panic!("TypeSpec cannot be resolved to TypeDef"),
        }
    }
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConstantType {
    Boolean = 0x02,
    Char = 0x03,
    Int8 = 0x04,
    UInt8 = 0x05,
    Int16 = 0x06,
    UInt16 = 0x07,
    Int32 = 0x08,
    UInt32 = 0x09,
    Int64 = 0x0a,
    UInt64 = 0x0b,
    Float32 = 0x0c,
    Float64 = 0x0d,
    String = 0x0e,
    Class = 0x12
}

#[derive(Copy, Clone, PartialEq)]
pub enum PrimitiveValue {
    Boolean(bool),
    Char(u16),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
}

impl<'db> fmt::Debug for PrimitiveValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PrimitiveValue::*;
        match self {
            Boolean(v) => write!(f, "bool({})", v),
            Char(v) => write!(f, "char({})", v),
            Int8(v) => write!(f, "int8({})", v),
            UInt8(v) => write!(f, "unsigned int8({})", v),
            Int16(v) => write!(f, "int16({})", v),
            UInt16(v) => write!(f, "unsigned int16({})", v),
            Int32(v) => write!(f, "int32({})", v),
            UInt32(v) => write!(f, "unsigned int32({})", v),
            Int64(v) => write!(f, "int64({})", v),
            UInt64(v) => write!(f, "unsigned int64({})", v),
            Float32(v) => write!(f, "float32({})", v),
            Float64(v) => write!(f, "float64({})", v)
        }
    }
}

// ECMA-335, II.16.2
#[derive(Copy, Clone, PartialEq)]
pub enum FieldInit<'db> {
    Primitive(PrimitiveValue),
    String(Option<&'db str>),
    NullRef
}

impl<'db> fmt::Debug for FieldInit<'db> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FieldInit::*;
        match self {
            Primitive(p) => write!(f, "{:?}", p),
            String(Some(v)) => write!(f, "{:?}", v),
            NullRef | String(None) => write!(f, "nullref"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypeCategory {
    Interface,
    Class,
    Enum,
    Struct,
    Delegate
}
