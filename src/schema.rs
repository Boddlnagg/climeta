use num_traits::FromPrimitive;
use byteorder::{ByteOrder, LittleEndian};

use crate::database::{TableKind, TableDesc, FixedSize2, FixedSize4, FixedSize8, DynamicSize, Database, Tables, CodedIndex};
use crate::database::{Col0, Col1, Col2, Col3, Col4, Col5};
use crate::table::{TableRow, TableRowIterator};
use crate::{Result, ByteView};

pub mod flags;

#[derive(Copy, Clone)] pub struct Module;
#[derive(Copy, Clone)] pub struct TypeRef;
#[derive(Copy, Clone)] pub struct TypeDef;
#[derive(Copy, Clone)] pub struct Field;
#[derive(Copy, Clone)] pub struct MethodDef;
#[derive(Copy, Clone)] pub struct Param;
#[derive(Copy, Clone)] pub struct InterfaceImpl;
#[derive(Copy, Clone)] pub struct MemberRef;
#[derive(Copy, Clone)] pub struct Constant;
#[derive(Copy, Clone)] pub struct CustomAttribute;
#[derive(Copy, Clone)] pub struct FieldMarshal;
#[derive(Copy, Clone)] pub struct DeclSecurity;
#[derive(Copy, Clone)] pub struct ClassLayout;
#[derive(Copy, Clone)] pub struct FieldLayout;
#[derive(Copy, Clone)] pub struct StandAloneSig;
#[derive(Copy, Clone)] pub struct EventMap;
#[derive(Copy, Clone)] pub struct Event;
#[derive(Copy, Clone)] pub struct PropertyMap;
#[derive(Copy, Clone)] pub struct Property;
#[derive(Copy, Clone)] pub struct MethodSemantics;
#[derive(Copy, Clone)] pub struct MethodImpl;
#[derive(Copy, Clone)] pub struct ModuleRef;
#[derive(Copy, Clone)] pub struct TypeSpec;
#[derive(Copy, Clone)] pub struct ImplMap;
#[derive(Copy, Clone)] pub struct FieldRVA;
#[derive(Copy, Clone)] pub struct Assembly;
#[derive(Copy, Clone)] pub struct AssemblyProcessor;
#[derive(Copy, Clone)] pub struct AssemblyOS;
#[derive(Copy, Clone)] pub struct AssemblyRef;
#[derive(Copy, Clone)] pub struct AssemblyRefProcessor;
#[derive(Copy, Clone)] pub struct AssemblyRefOS;
#[derive(Copy, Clone)] pub struct File;
#[derive(Copy, Clone)] pub struct ExportedType;
#[derive(Copy, Clone)] pub struct ManifestResource;
#[derive(Copy, Clone)] pub struct NestedClass;
#[derive(Copy, Clone)] pub struct GenericParam;
#[derive(Copy, Clone)] pub struct MethodSpec;
#[derive(Copy, Clone)] pub struct GenericParamConstraint;

impl TableKind for Assembly {}
impl TableDesc for Assembly {
    type Columns = (FixedSize4, FixedSize8, FixedSize4, DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for AssemblyOS {}
impl TableDesc for AssemblyOS {
    type Columns = (FixedSize4, FixedSize4, FixedSize4);
}
impl TableKind for AssemblyProcessor {}
impl TableDesc for AssemblyProcessor {
    type Columns = (FixedSize4,);
}
impl TableKind for AssemblyRef {}
impl TableDesc for AssemblyRef {
    type Columns = (FixedSize8, FixedSize4, DynamicSize, DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for AssemblyRefOS {}
impl TableDesc for AssemblyRefOS {
    type Columns = (FixedSize4, FixedSize4, FixedSize4, DynamicSize);
}
impl TableKind for AssemblyRefProcessor {}
impl TableDesc for AssemblyRefProcessor {
    type Columns = (FixedSize4, DynamicSize);
}
impl TableKind for ClassLayout {}
impl TableDesc for ClassLayout {
    type Columns = (FixedSize2, FixedSize4, DynamicSize);
}
impl TableKind for Constant {}
impl TableDesc for Constant {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableKind for CustomAttribute {}
impl TableDesc for CustomAttribute {
    type Columns = (DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for DeclSecurity {}
impl TableDesc for DeclSecurity {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableKind for EventMap {}
impl TableDesc for EventMap {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableKind for Event {}
impl TableDesc for Event {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableKind for ExportedType {}
impl TableDesc for ExportedType {
    type Columns = (FixedSize4, FixedSize4, DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for Field {}
impl TableDesc for Field {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableKind for FieldLayout {}
impl TableDesc for FieldLayout {
    type Columns = (FixedSize4, DynamicSize);
}
impl TableKind for FieldMarshal {}
impl TableDesc for FieldMarshal {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableKind for FieldRVA {}
impl TableDesc for FieldRVA {
    type Columns = (FixedSize4, DynamicSize);
}
impl TableKind for File {}
impl TableDesc for File {
    type Columns = (FixedSize4, DynamicSize, DynamicSize);
}
impl TableKind for GenericParam {}
impl TableDesc for GenericParam {
    type Columns = (FixedSize2, FixedSize2, DynamicSize, DynamicSize);
}
impl TableKind for GenericParamConstraint {}
impl TableDesc for GenericParamConstraint {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableKind for ImplMap {}
impl TableDesc for ImplMap {
    type Columns = (FixedSize2, DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for InterfaceImpl {}
impl TableDesc for InterfaceImpl {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableKind for ManifestResource {}
impl TableDesc for ManifestResource {
    type Columns = (FixedSize4, FixedSize4, DynamicSize, DynamicSize);
}
impl TableKind for MemberRef {}
impl TableDesc for MemberRef {
    type Columns = (DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for MethodDef {}
impl TableDesc for MethodDef {
    type Columns = (FixedSize4, FixedSize2, FixedSize2, DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for MethodImpl {}
impl TableDesc for MethodImpl {
    type Columns = (DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for MethodSemantics {}
impl TableDesc for MethodSemantics {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableKind for MethodSpec {}
impl TableDesc for MethodSpec {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableKind for Module {}
impl TableDesc for Module {
    type Columns = (FixedSize2, DynamicSize, DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for ModuleRef {}
impl TableDesc for ModuleRef {
    type Columns = (DynamicSize,);
}
impl TableKind for NestedClass {}
impl TableDesc for NestedClass {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableKind for Param {}
impl TableDesc for Param {
    type Columns = (FixedSize2, FixedSize2, DynamicSize);
}
impl TableKind for Property {}
impl TableDesc for Property {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableKind for PropertyMap {}
impl TableDesc for PropertyMap {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableKind for StandAloneSig {}
impl TableDesc for StandAloneSig {
    type Columns = (DynamicSize,);
}
impl TableKind for TypeDef {}
impl TableDesc for TypeDef {
    type Columns = (FixedSize4, DynamicSize, DynamicSize, DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for TypeRef {}
impl TableDesc for TypeRef {
    type Columns = (DynamicSize, DynamicSize, DynamicSize);
}
impl TableKind for TypeSpec {}
impl TableDesc for TypeSpec {
    type Columns = (DynamicSize,);
}

macro_rules! coded_index {
    ($name:ident[$bits:tt] { $($n:tt => $ty:ident),+ }) => {
        pub enum $name<'db> {
            $($ty(TableRow<'db, $ty>)),+
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
                if $(Self::needs_4byte_index(tables.get_table_info::<$ty>().m_row_count, $bits))||+ {
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

impl<'db> TableRow<'db, AssemblyRef> {
    pub fn public_key_or_token(&self) -> Result<&'db [u8]> {
        self.get_blob::<Col2>()
    }

    pub fn name(&self) -> Result<&'db str> {
        self.get_string::<Col3>()
    }

    pub fn culture(&self) -> Result<&'db str> {
        self.get_string::<Col4>()
    }

    pub fn hash_value(&self) -> Result<&'db str> {
        self.get_string::<Col5>()
    }
}

impl<'db> TableRow<'db, Constant> {
    pub fn typ(&self) -> Result<ConstantType> {
        <ConstantType as FromPrimitive>::from_u16(self.get_value::<Col0, _>()?).ok_or_else(|| "Invalid ConstantType".into())
    }

    pub fn parent(&self) -> Result<Option<HasConstant<'db>>> {
        self.get_coded_index::<Col1, HasConstant>()
    }

    pub fn value(&self) -> Result<ConstantValue> {
        use ConstantValue::*;
        let bytes = self.get_blob::<Col2>()?;
        Ok(match self.typ()? {
            ConstantType::Boolean => Boolean(bytes[0] != 0),
            ConstantType::Char => Char(LittleEndian::read_u16(bytes)),
            ConstantType::Int8 => Int8(bytes[0] as i8),
            ConstantType::UInt8 => UInt8(bytes[0]),
            ConstantType::Int16 => Int16(LittleEndian::read_i16(bytes)),
            ConstantType::UInt16 => UInt16(LittleEndian::read_u16(bytes)),
            ConstantType::Int32 => Int32(LittleEndian::read_i32(bytes)),
            ConstantType::UInt32 => UInt32(LittleEndian::read_u32(bytes)),
            ConstantType::Int64 => Int64(LittleEndian::read_i64(bytes)),
            ConstantType::UInt64 => UInt64(LittleEndian::read_u64(bytes)),
            ConstantType::Float32 => Float32(LittleEndian::read_f32(bytes)),
            ConstantType::Float64 => Float64(LittleEndian::read_f64(bytes)),
            ConstantType::String => {
                let string = match bytes.as_string(0) {
                    None => None,
                    Some(s) => Some(std::str::from_utf8(s)?)
                };
                String(string)
            },
            ConstantType::Class => {
                assert_eq!(LittleEndian::read_u32(bytes), 0);
                Class // nullref
            }
        })
    }
}

impl<'db> TableRow<'db, Event> {
    pub fn event_flags(&self) -> Result<flags::EventAttributes> {
        Ok(flags::EventAttributes(self.get_value::<Col0, _>()?))
    }
}

impl<'db> TableRow<'db, Field> {
    pub fn flags(&self) -> Result<flags::FieldAttributes> {
        Ok(flags::FieldAttributes(self.get_value::<Col0, _>()?))
    }

    pub fn name(&self) -> Result<&'db str> {
        self.get_string::<Col1>()
    }
}

impl<'db> TableRow<'db, GenericParam> {
    pub fn number(&self) -> Result<u16> {
        self.get_value::<Col0, _>()
    }

    pub fn flags(&self) -> Result<flags::GenericParamAttributes> {
        Ok(flags::GenericParamAttributes(self.get_value::<Col1, _>()?))
    }

    pub fn owner(&self) -> Result<Option<TypeOrMethodDef<'db>>> {
        self.get_coded_index::<Col2, TypeOrMethodDef>()
    }

    pub fn name(&self) -> Result<&'db str> {
        self.get_string::<Col3>()
    }
}

impl<'db> TableRow<'db, MethodDef> {
    pub fn rva(&self) -> Result<u32> {
        self.get_value::<Col0, _>()
    }
    pub fn impl_flags(&self) -> Result<flags::MethodImplAttributes> {
        Ok(flags::MethodImplAttributes(self.get_value::<Col1, _>()?))
    }

    pub fn flags(&self) -> Result<flags::MethodAttributes> {
        Ok(flags::MethodAttributes(self.get_value::<Col2, _>()?))
    }

    pub fn name(&self) -> Result<&'db str> {
        self.get_string::<Col3>()
    }

    pub fn param_list(&self) -> Result<TableRowIterator<'db, Param>> {
        self.get_list::<Col5, Param>()
    }
}

impl<'db> TableRow<'db, MethodSemantics> {
    pub fn semantics(&self) -> Result<flags::MethodSemanticsAttributes> {
        Ok(flags::MethodSemanticsAttributes(self.get_value::<Col0, _>()?))
    }

    pub fn method(&self) -> Result<TableRow<'db, MethodDef>> {
        self.get_target_row::<Col1, MethodDef>()
    }

    pub fn association(&self) -> Result<Option<HasSemantics<'db>>> {
        self.get_coded_index::<Col2, HasSemantics>()
    }
}

impl<'db> TableRow<'db, Param> {
    pub fn flags(&self) -> Result<flags::ParamAttributes> {
        Ok(flags::ParamAttributes(self.get_value::<Col0, _>()?))
    }

    pub fn sequence(&self) -> Result<u16> {
        self.get_value::<Col1, u16>()
    }

    pub fn name(&self) -> Result<&'db str> {
        self.get_string::<Col2>()
    }
}

impl<'db> TableRow<'db, Property> {
    pub fn flags(&self) -> Result<flags::PropertyAttributes> {
        Ok(flags::PropertyAttributes(self.get_value::<Col0, _>()?))
    }
}

impl<'db> TableRow<'db, TypeDef> {
    pub fn flags(&self) -> Result<flags::TypeAttributes> {
        Ok(flags::TypeAttributes(self.get_value::<Col0, _>()?))
    }

    pub fn type_name(&self) -> Result<&'db str> {
        self.get_string::<Col1>()
    }

    pub fn type_namespace(&self) -> Result<&'db str> {
        self.get_string::<Col2>()
    }

    pub fn extends(&self) -> Result<Option<TypeDefOrRef<'db>>> {
        self.get_coded_index::<Col3, TypeDefOrRef>()
    }

    pub fn field_list(&self) -> Result<TableRowIterator<'db, Field>> {
        self.get_list::<Col4, Field>()
    }

    pub fn method_list(&self) -> Result<TableRowIterator<'db, MethodDef>> {
        self.get_list::<Col5, MethodDef>()
    }
}

impl<'db> TableRow<'db, TypeRef> {
    pub fn resolution_scope(&self) -> Result<Option<ResolutionScope<'db>>> {
        self.get_coded_index::<Col0, ResolutionScope>()
    }

    pub fn type_name(&self) -> Result<&'db str> {
        self.get_string::<Col1>()
    }

    pub fn type_namespace(&self) -> Result<&'db str> {
        self.get_string::<Col2>()
    }
}
