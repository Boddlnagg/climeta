use num_traits::FromPrimitive;
use byteorder::{ByteOrder, LittleEndian};

use crate::database::{TableDesc, FixedSize2, FixedSize4, FixedSize8, DynamicSize, Database, Tables, CodedIndex};
use crate::database::{Col0, Col1, Col2, Col3, Col4, Col5};
use crate::table::{TableRow, TableRowIterator};
use crate::{Result, ByteView};

pub struct Module;
pub struct TypeRef;
pub struct TypeDef;
pub struct Field;
pub struct MethodDef;
pub struct Param;
pub struct InterfaceImpl;
pub struct MemberRef;
pub struct Constant;
pub struct CustomAttribute;
pub struct FieldMarshal;
pub struct DeclSecurity;
pub struct ClassLayout;
pub struct FieldLayout;
pub struct StandAloneSig;
pub struct EventMap;
pub struct Event;
pub struct PropertyMap;
pub struct Property;
pub struct MethodSemantics;
pub struct MethodImpl;
pub struct ModuleRef;
pub struct TypeSpec;
pub struct ImplMap;
pub struct FieldRVA;
pub struct Assembly;
pub struct AssemblyProcessor;
pub struct AssemblyOS;
pub struct AssemblyRef;
pub struct AssemblyRefProcessor;
pub struct AssemblyRefOS;
pub struct File;
pub struct ExportedType;
pub struct ManifestResource;
pub struct NestedClass;
pub struct GenericParam;
pub struct MethodSpec;
pub struct GenericParamConstraint;

impl TableDesc for Assembly {
    type Columns = (FixedSize4, FixedSize8, FixedSize4, DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for AssemblyOS {
    type Columns = (FixedSize4, FixedSize4, FixedSize4);
}
impl TableDesc for AssemblyProcessor {
    type Columns = (FixedSize4,);
}
impl TableDesc for AssemblyRef {
    type Columns = (FixedSize8, FixedSize4, DynamicSize, DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for AssemblyRefOS {
    type Columns = (FixedSize4, FixedSize4, FixedSize4, DynamicSize);
}
impl TableDesc for AssemblyRefProcessor {
    type Columns = (FixedSize4, DynamicSize);
}
impl TableDesc for ClassLayout {
    type Columns = (FixedSize2, FixedSize4, DynamicSize);
}
impl TableDesc for Constant {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableDesc for CustomAttribute {
    type Columns = (DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for DeclSecurity {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableDesc for EventMap {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableDesc for Event {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableDesc for ExportedType {
    type Columns = (FixedSize4, FixedSize4, DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for Field {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableDesc for FieldLayout {
    type Columns = (FixedSize4, DynamicSize);
}
impl TableDesc for FieldMarshal {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableDesc for FieldRVA {
    type Columns = (FixedSize4, DynamicSize);
}
impl TableDesc for File {
    type Columns = (FixedSize4, DynamicSize, DynamicSize);
}
impl TableDesc for GenericParam {
    type Columns = (FixedSize2, FixedSize2, DynamicSize, DynamicSize);
}
impl TableDesc for GenericParamConstraint {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableDesc for ImplMap {
    type Columns = (FixedSize2, DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for InterfaceImpl {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableDesc for ManifestResource {
    type Columns = (FixedSize4, FixedSize4, DynamicSize, DynamicSize);
}
impl TableDesc for MemberRef {
    type Columns = (DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for MethodDef {
    type Columns = (FixedSize4, FixedSize2, FixedSize2, DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for MethodImpl {
    type Columns = (DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for MethodSemantics {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableDesc for MethodSpec {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableDesc for Module {
    type Columns = (FixedSize2, DynamicSize, DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for ModuleRef {
    type Columns = (DynamicSize,);
}
impl TableDesc for NestedClass {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableDesc for Param {
    type Columns = (FixedSize2, FixedSize2, DynamicSize);
}
impl TableDesc for Property {
    type Columns = (FixedSize2, DynamicSize, DynamicSize);
}
impl TableDesc for PropertyMap {
    type Columns = (DynamicSize, DynamicSize);
}
impl TableDesc for StandAloneSig {
    type Columns = (DynamicSize,);
}
impl TableDesc for TypeDef {
    type Columns = (FixedSize4, DynamicSize, DynamicSize, DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for TypeRef {
    type Columns = (DynamicSize, DynamicSize, DynamicSize);
}
impl TableDesc for TypeSpec {
    type Columns = (DynamicSize,);
}

macro_rules! coded_index {
    ($name:ident[$bits:tt] { $($n:tt => $ty:ident),+ }) => {
        pub enum $name<'t> {
            $($ty(TableRow<'t, $ty>)),+
        }

        impl<'t> CodedIndex for $name<'t> {
            type Tables = &'t Tables<'t>;
            fn decode(idx: u32, tables: Self::Tables) -> Result<Option<Self>> {
                let tag = idx & ((1 << $bits) - 1);
                let row = idx >> $bits as u32;
                if row == 0 {
                    return Ok(None);
                }
                let row = row - 1;
                Ok(Some(match tag {
                    $($n => $name::$ty(tables.get_table::<$ty>().get_row(row)?),)+
                    _ => unreachable!()
                }))
            }

            fn index_size(tables: Self::Tables) -> DynamicSize {
                if $(Self::needs_4byte_index(tables.get_table::<$ty>().size(), $bits))||+ {
                    DynamicSize::Size4
                } else {
                    DynamicSize::Size2
                }
            }
        }

        impl <'t: 'b, 'b> std::fmt::Debug for $name<'t> {
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
pub enum ConstantValue<'a> {
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
    String(Option<&'a str>),
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

impl<'t> TableRow<'t, AssemblyRef> {
    pub fn public_key_or_token(&self, db: &'t Database<'t>) -> Result<&'t [u8]> {
        self.get_blob::<Col2>(db)
    }

    pub fn name(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col3>(db)
    }

    pub fn culture(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col4>(db)
    }

    pub fn hash_value(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col5>(db)
    }
}

impl<'t> TableRow<'t, Constant> {
    pub fn typ(&self) -> Result<ConstantType> {
        <ConstantType as FromPrimitive>::from_u16(self.get_value::<Col0, _>()?).ok_or_else(|| "Invalid ConstantType".into())
    }

    pub fn parent(&self, db: &'t Database<'t>) -> Result<Option<HasConstant<'t>>> {
        self.get_coded_index::<Col1, HasConstant>(&db.m_tables)
    }

    pub fn value(&self, db: &'t Database<'t>) -> Result<ConstantValue> {
        use ConstantValue::*;
        let bytes = self.get_blob::<Col2>(db)?;
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

impl<'t> TableRow<'t, Field> {
    pub fn name(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col1>(db)
    }
}

impl<'t> TableRow<'t, MethodDef> {
    pub fn rva(&self) -> Result<u32> {
        self.get_value::<Col0, _>()
    }

    pub fn name(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col3>(db)
    }

    pub fn param_list(&self, db: &'t Database<'t>) -> Result<TableRowIterator<'t, Param>> {
        self.get_list::<Col5, Param>(&db.m_tables)
    }
}

impl<'t> TableRow<'t, Param> {
    pub fn sequence(&self) -> Result<u16> {
        self.get_value::<Col1, u16>()
    }

    pub fn name(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col2>(db)
    }
}

impl<'t> TableRow<'t, TypeDef> {
    pub fn type_name(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col1>(db)
    }

    pub fn type_namespace(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col2>(db)
    }

    pub fn extends(&self, db: &'t Database<'t>) -> Result<Option<TypeDefOrRef<'t>>> {
        self.get_coded_index::<Col3, TypeDefOrRef>(&db.m_tables)
    }

    pub fn field_list(&self, db: &'t Database<'t>) -> Result<TableRowIterator<'t, Field>> {
        self.get_list::<Col4, Field>(&db.m_tables)
    }

    pub fn method_list(&self, db: &'t Database<'t>) -> Result<TableRowIterator<'t, MethodDef>> {
        self.get_list::<Col5, MethodDef>(&db.m_tables)
    }
}

impl<'t> TableRow<'t, TypeRef> {
    pub fn resolution_scope(&self, db: &'t Database<'t>) -> Result<Option<ResolutionScope<'t>>> {
        self.get_coded_index::<Col0, ResolutionScope>(&db.m_tables)
    }

    pub fn type_name(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col1>(db)
    }

    pub fn type_namespace(&self, db: &'t Database<'t>) -> Result<&'t str> {
        self.get_string::<Col2>(db)
    }
}
