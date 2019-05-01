use crate::database::{Database, Tables, CodedIndex};
use crate::Result;

use crate::core::columns::DynamicSize;

pub mod flags;
mod rows;
pub use rows::*;

macro_rules! table_kind {
    ($ty:ident [$($colty:ty),+]) => {
        #[derive(Copy, Clone)]
        pub struct $ty;

        impl TableKind for $ty {
            // unfortunately no generic associated type Row<'db> yet ...
        }

        impl<'a> super::TableRowAccess for &'a $ty {
            type Table = Table<'a, $ty>;
            type Out = super::rows::$ty<'a>;

            fn get(table: Self::Table, row: u32) -> Self::Out {
                super::rows::$ty(Row::new(table, row))
            }
        }

        impl TableDesc for $ty {
            type Columns = ($($colty),+ ,);
        }
    }
}

pub mod marker {
    use crate::core::table::{Table, Row};
    use crate::database::{TableKind, TableDesc};
    use crate::core::columns::{FixedSize2, FixedSize4, FixedSize8, DynamicSize};

    table_kind!(Assembly [FixedSize4, FixedSize8, FixedSize4, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(AssemblyOS [FixedSize4, FixedSize4, FixedSize4]);
    table_kind!(AssemblyProcessor [FixedSize4]);
    table_kind!(AssemblyRef [FixedSize8, FixedSize4, DynamicSize, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(AssemblyRefOS [FixedSize4, FixedSize4, FixedSize4, DynamicSize]);
    table_kind!(AssemblyRefProcessor [FixedSize4, DynamicSize]);
    table_kind!(ClassLayout [FixedSize2, FixedSize4, DynamicSize]);
    table_kind!(Constant [FixedSize2, DynamicSize, DynamicSize]);
    table_kind!(CustomAttribute [DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(DeclSecurity [FixedSize2, DynamicSize, DynamicSize]);
    table_kind!(Event [FixedSize2, DynamicSize, DynamicSize]);
    table_kind!(EventMap [DynamicSize, DynamicSize]);
    table_kind!(ExportedType [FixedSize4, FixedSize4, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(Field [FixedSize2, DynamicSize, DynamicSize]);
    table_kind!(FieldLayout [FixedSize4, DynamicSize]);
    table_kind!(FieldMarshal [DynamicSize, DynamicSize]);
    table_kind!(FieldRVA [FixedSize4, DynamicSize]);
    table_kind!(File [FixedSize4, DynamicSize, DynamicSize]);
    table_kind!(GenericParam [FixedSize2, FixedSize2, DynamicSize, DynamicSize]);
    table_kind!(GenericParamConstraint [DynamicSize, DynamicSize]);
    table_kind!(ImplMap [FixedSize2, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(InterfaceImpl [DynamicSize, DynamicSize]);
    table_kind!(ManifestResource [FixedSize4, FixedSize4, DynamicSize, DynamicSize]);
    table_kind!(MemberRef [DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(MethodDef [FixedSize4, FixedSize2, FixedSize2, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(MethodImpl [DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(MethodSemantics [FixedSize2, DynamicSize, DynamicSize]);
    table_kind!(MethodSpec [DynamicSize, DynamicSize]);
    table_kind!(Module [FixedSize2, DynamicSize, DynamicSize, DynamicSize, DynamicSize]);
    table_kind!(ModuleRef [DynamicSize]);
    table_kind!(NestedClass [DynamicSize, DynamicSize]);
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
        pub enum $name<'db> {
            $($ty(rows::$ty<'db>)),+
        }

        impl<'db> CodedIndex for $name<'db> {
            type Database = &'db Database<'db>;
            type Tables = &'db Tables<'db>;

            fn decode(idx: u32, db: Self::Database) -> Result<Option<Self>> {
                let tag = idx & ((1 << $bits) - 1);
                let row = idx >> $bits as u32;
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
                if $(Self::needs_4byte_index(tables.get_table_info::<marker::$ty>().m_row_count, $bits))||+ {
                    DynamicSize::Size4
                } else {
                    DynamicSize::Size2
                }
            }
        }

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

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
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

#[derive(Debug, Copy, Clone)]
pub enum ConstantValue<'db> {
    Boolean(bool),
    Char(u16),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32) ,
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    String(Option<&'db str>),
    Class
}

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
enum ElementType {
    End = 0x00, // Sentinel value

    Void = 0x01,
    Boolean = 0x02,
    Char = 0x03,
    I1 = 0x04,
    U1 = 0x05,
    I2 = 0x06,
    U2 = 0x07,
    I4 = 0x08,
    U4 = 0x09,
    I8 = 0x0a,
    U8 = 0x0b,
    R4 = 0x0c,
    R8 = 0x0d,
    String = 0x0e,

    Ptr = 0x0f, // Followed by TypeSig
    ByRef = 0x10, // Followed by TypeSig
    ValueType = 0x11, // Followed by TypeDef or TypeRef
    Class = 0x12, // Followed by TypeDef or TypeRef
    Var = 0x13, // Generic parameter in a type definition, represented as unsigned integer
    Array = 0x14,
    GenericInst = 0x15,
    TypedByRef = 0x16,

    I = 0x18, // System.IntPtr
    U = 0x19, // System.UIntPtr

    FnPtr = 0x1b, // Followed by full method signature
    Object = 0x1c, // System.Object
    SZArray = 0x1d,
    MVar = 0x1e, // Generic parameter in a method definition, represented as unsigned integer
    CModReqd = 0x1f, // Required modifier, followed by a TypeDef or TypeRef
    CModOpt = 0x20, // Optional modifier, followed by a TypeDef or TypeRef
    Internal = 0x21,

    Modifier = 0x40, // Or'd with folowing element types
    Sentinel = 0x41, // Sentinel for vararg method signature

    Pinned = 0x45,

    Type = 0x50, // System.Type
    TaggedObject = 0x51, // Boxed object (in custom attributes)
    Field = 0x53, // Custom attribute field
    Property = 0x54, // Custom attribute property
    Enum = 0x55, // Custom attribute enum
}
