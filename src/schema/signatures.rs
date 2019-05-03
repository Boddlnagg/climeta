use std::io::Cursor;
use std::fmt;
use std::mem;
use byteorder::ReadBytesExt;
use crate::Result;
use crate::core::db::{Database, CodedIndex};
use super::TypeDefOrRef;

trait ByteCursorExt {
    type Out;
    fn bytes_left(&self) -> Self::Out;
}

impl<'a> ByteCursorExt for Cursor<&'a [u8]> {
    type Out = &'a [u8];
    fn bytes_left(&self) -> Self::Out {
        &self.get_ref()[self.position() as usize ..]
    }
}

fn uncompress_unsigned(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let first = cursor.read_u8()?;
    if (first & 0x80) == 0x00 {
        Ok(first as u32)
    } else if (first & 0xc0) == 0x80 {
        let mut value = ((first & 0x3f) as u32) << 8;
        value |= cursor.read_u8()? as u32;
        Ok(value)
    } else if (first & 0xe0) == 0xc0 {
        let mut value = ((first & 0x1f) as u32) << 24;
        value |= (cursor.read_u8()? as u32) << 16;
        value |= (cursor.read_u8()? as u32) << 8;
        value |= cursor.read_u8()? as u32;
        Ok(value)
    } else {
        Err("Invalid compressed integer in blob".into())
    }
}

#[allow(dead_code, unused_variables)]
fn uncompress_signed(cursor: &mut Cursor<&[u8]>) -> Result<i32> {
    unimplemented!()
}


#[allow(non_upper_case_globals, dead_code)]
mod bits {
    pub const CallingConvention_mask: u8 = 0x15; // 10101
    pub const DEFAULT: u8 = 0x00;
    pub const VARARG: u8 = 0x05;
    pub const FIELD: u8 = 0x06;
    pub const GENERIC: u8 = 0x10;

    pub const HASTHIS: u8 = 0x20;
    pub const EXPLICITTHIS: u8 = 0x40;

    pub const ELEMENT_TYPE_END: u8 = 0x00;
    pub const ELEMENT_TYPE_VOID: u8 = 0x01;
    pub const ELEMENT_TYPE_BOOLEAN: u8 = 0x02;
    pub const ELEMENT_TYPE_CHAR: u8 = 0x03;
    pub const ELEMENT_TYPE_I1: u8 = 0x04;
    pub const ELEMENT_TYPE_U1: u8 = 0x05;
    pub const ELEMENT_TYPE_I2: u8 = 0x06;
    pub const ELEMENT_TYPE_U2: u8 = 0x07;
    pub const ELEMENT_TYPE_I4: u8 = 0x08;
    pub const ELEMENT_TYPE_U4: u8 = 0x09;
    pub const ELEMENT_TYPE_I8: u8 = 0x0a;
    pub const ELEMENT_TYPE_U8: u8 = 0x0b;
    pub const ELEMENT_TYPE_R4: u8 = 0x0c;
    pub const ELEMENT_TYPE_R8: u8 = 0x0d;
    pub const ELEMENT_TYPE_STRING: u8 = 0x0e;
    pub const ELEMENT_TYPE_PTR: u8 = 0x0f;
    pub const ELEMENT_TYPE_BYREF: u8 = 0x10;
    pub const ELEMENT_TYPE_VALUETYPE: u8 = 0x11;
    pub const ELEMENT_TYPE_CLASS: u8 = 0x12;
    pub const ELEMENT_TYPE_VAR: u8 = 0x13;
    pub const ELEMENT_TYPE_ARRAY: u8 = 0x14;
    pub const ELEMENT_TYPE_GENERICINST: u8 = 0x15;
    pub const ELEMENT_TYPE_TYPEDBYREF: u8 = 0x16;
    pub const ELEMENT_TYPE_I: u8 = 0x18;
    pub const ELEMENT_TYPE_U: u8 = 0x19;
    pub const ELEMENT_TYPE_FNPTR: u8 = 0x1b;
    pub const ELEMENT_TYPE_OBJECT: u8 = 0x1c;
    pub const ELEMENT_TYPE_SZARRAY: u8 = 0x1d;
    pub const ELEMENT_TYPE_MVAR: u8 = 0x1e;
    pub const ELEMENT_TYPE_CMOD_REQD: u8 = 0x1f;
    pub const ELEMENT_TYPE_CMOD_OPT: u8 = 0x20;
    pub const ELEMENT_TYPE_INTERNAL: u8 = 0x21;
    pub const ELEMENT_TYPE_MODIFIER: u8 = 0x40;
    pub const ELEMENT_TYPE_SENTINEL: u8 = 0x41;
    pub const ELEMENT_TYPE_PINNED: u8 = 0x45;
    // 0x50 (System.Type)
    // 0x51 (Boxed object in custom attributes)
    // 0x52 (Reserved)
    // 0x53 (FIELD in custom attributes)
    // 0x54 (PROPERTY in custom attributes)
    // 0x55 (enum in custom attributes)
}

// II.23.2.1
pub struct MethodDefSig<'db> {
    m_initial_byte: u8,
    m_generic_param_count: u32,
    m_ret_type: RetType<'db>,
    m_params: Box<[ParamSig<'db>]>, // TODO: iterator?
}

impl<'db> MethodDefSig<'db> {
    pub(crate) fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<MethodDefSig<'db>> {
        let initial_byte = cur.read_u8()?;
        let generic_param_count = if initial_byte & bits::GENERIC != 0 {
            uncompress_unsigned(cur)?
        } else {
            0
        };

        let param_count = uncompress_unsigned(cur)?;

        let ret_type = RetType::parse(cur, db)?;

        let mut params = Vec::with_capacity(param_count as usize);
        
        for _ in 0..param_count {
            params.push(ParamSig::parse(cur, db)?);
        }

        Ok(MethodDefSig {
            m_initial_byte: initial_byte,
            m_generic_param_count: generic_param_count,
            m_ret_type: ret_type,
            m_params: params.into_boxed_slice()
        })
    }

    pub fn has_this(&self) -> bool {
        self.m_initial_byte & bits::HASTHIS != 0
    }

    pub fn explicit_this(&self) -> bool {
        self.m_initial_byte & bits::EXPLICITTHIS != 0
    }

    pub fn is_generic(&self) -> bool {
        self.m_generic_param_count != 0
    }

    pub fn is_vararg(&self) -> bool {
        self.m_initial_byte & bits::VARARG != 0
    }

    pub fn generic_param_count(&self) -> u32 {
        self.m_generic_param_count
    }

    pub fn return_type(&self) -> &RetType<'db> {
        &self.m_ret_type
    }
    
    pub fn params(&self) -> &[ParamSig<'db>] {
        &self.m_params
    }
}

// TODO: impl Debug for MethodDefSig (s.a. II.15.3)

// TODO: this could also internally be Box<(Type, [CustomMod])>,
//       where the tuple is dynamically sized, to have only one dynamic allocation
pub struct Array<'db> {
    m_type: Box<Type<'db>>,
    m_cmod: Vec<CustomMod<'db>>
    // TODO: optional ArrayShape
}

impl<'db> Array<'db> {
    fn parse_szarray(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<Array<'db>> {
        // ELEMENT_TYPE_SZARRAY already consumed
        let cmod = CustomMod::parse(cur, db)?;
        Ok(Array {
            m_type: Box::new(Type::parse(cur, db)?),
            m_cmod: cmod
        })
    }

    pub fn elem_type(&self) -> &Type<'db> {
        &self.m_type
    }

    pub fn custom_mod(&self) -> &[CustomMod<'db>] {
        &self.m_cmod[..]
    }
}


impl<'db> fmt::Debug for Array<'db> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: is this correct?
        write!(f, "{:?}[]", self.m_type)
    }
}


#[derive(Copy, Clone)]
pub enum TypeTag {
    Class,
    ValueType
}

impl<'db> fmt::Debug for TypeTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeTag::ValueType => write!(f, "valuetype"),
            TypeTag::Class => write!(f, "class")
        }
    }
}

#[derive(Copy, Clone)]
pub enum GenericVarScope {
    Type,
    Method
}

#[derive(Copy, Clone)]
pub enum PrimitiveType {
    Boolean,
    Char,
    I1,
    U1,
    I2,
    U2,
    I4,
    U4,
    I8,
    U8,
    R4,
    R8,
    I,
    U,
}

impl fmt::Debug for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // ECMA-335, II.7.1
        use PrimitiveType::*;
        match *self {
            Boolean => write!(f, "bool"),
            Char => write!(f, "char"),
            I1 => write!(f, "int8"),
            U1 => write!(f, "unsigned int8"),
            I2 => write!(f, "int16"),
            U2 => write!(f, "unsigned int16"),
            I4 => write!(f, "int32"),
            U4 => write!(f, "unsigned int32"),
            I8 => write!(f, "int64"),
            U8 => write!(f, "unsigned int64"),
            R4 => write!(f, "float32"),
            R8 => write!(f, "float64"),
            I => write!(f, "native int"),
            U => write!(f, "native unsigned int")
        }
    }
}

// II.23.2.12
pub enum Type<'db> {
    Primitive(PrimitiveType),
    Array(Array<'db>), // for ARRAY and SZARRAY
    Ref(TypeTag, TypeDefOrRef<'db>, Option<Box<[Type<'db>]>>),
    //FnPtr, // TODO
    GenericVar(GenericVarScope, u32),
    Object,
    //Ptr, // TODO
    String,
}

impl<'db> Type<'db> {
    fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<Type<'db>> {
        let element_type = uncompress_unsigned(cur)?;
        Ok(match element_type as u8 {
            bits::ELEMENT_TYPE_BOOLEAN => Type::Primitive(PrimitiveType::Boolean),
            bits::ELEMENT_TYPE_CHAR => Type::Primitive(PrimitiveType::Char),
            bits::ELEMENT_TYPE_I1 => Type::Primitive(PrimitiveType::I1),
            bits::ELEMENT_TYPE_U1 => Type::Primitive(PrimitiveType::U1),
            bits::ELEMENT_TYPE_I2 => Type::Primitive(PrimitiveType::I2),
            bits::ELEMENT_TYPE_U2 => Type::Primitive(PrimitiveType::U2),
            bits::ELEMENT_TYPE_I4 => Type::Primitive(PrimitiveType::I4),
            bits::ELEMENT_TYPE_U4 => Type::Primitive(PrimitiveType::U4),
            bits::ELEMENT_TYPE_I8 => Type::Primitive(PrimitiveType::I8),
            bits::ELEMENT_TYPE_U8 => Type::Primitive(PrimitiveType::U8),
            bits::ELEMENT_TYPE_R4 => Type::Primitive(PrimitiveType::R4),
            bits::ELEMENT_TYPE_R8 => Type::Primitive(PrimitiveType::R8),
            bits::ELEMENT_TYPE_I => Type::Primitive(PrimitiveType::I),
            bits::ELEMENT_TYPE_U => Type::Primitive(PrimitiveType::U),
            bits::ELEMENT_TYPE_ARRAY => unimplemented!(),
            bits::ELEMENT_TYPE_CLASS => Type::Ref(TypeTag::Class, TypeDefOrRef::decode(uncompress_unsigned(cur)?, db)?.expect("Null type in Class Type"), None),
            bits::ELEMENT_TYPE_FNPTR => unimplemented!(),
            bits::ELEMENT_TYPE_GENERICINST => {
                let (typetag, typ, args) = parse_generic_inst(cur, db)?;
                Type::Ref(typetag, typ, Some(args))
            },
            bits::ELEMENT_TYPE_MVAR => Type::GenericVar(GenericVarScope::Method, uncompress_unsigned(cur)?),
            bits::ELEMENT_TYPE_OBJECT => Type::Object,
            bits::ELEMENT_TYPE_PTR => unimplemented!(),
            bits::ELEMENT_TYPE_STRING => Type::String,
            bits::ELEMENT_TYPE_SZARRAY => Type::Array(Array::parse_szarray(cur, db)?),
            bits::ELEMENT_TYPE_VALUETYPE => Type::Ref(TypeTag::ValueType, TypeDefOrRef::decode(uncompress_unsigned(cur)?, db)?.expect("Null type in ValueType Type"), None),
            bits::ELEMENT_TYPE_VAR => Type::GenericVar(GenericVarScope::Type, uncompress_unsigned(cur)?),
            b => return Err(format!("Unexpected element type for Type: {}", b).into())
        })
    }
}

fn parse_generic_inst<'db>(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<(TypeTag, TypeDefOrRef<'db>, Box<[Type<'db>]>)> {
    let typetag = match uncompress_unsigned(cur)? as u8 {
        bits::ELEMENT_TYPE_CLASS => TypeTag::Class,
        bits::ELEMENT_TYPE_VALUETYPE => TypeTag::ValueType,
        _ => return Err("Generic type instantiation signatures must begin with either ELEMENT_TYPE_CLASS or ELEMENT_TYPE_VALUE".into())
    };

    let typ = TypeDefOrRef::decode(uncompress_unsigned(cur)?, db)?.expect("Null type in GenericInst arg");
    let arg_count = uncompress_unsigned(cur)?;
    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        args.push(Type::parse(cur, db)?);
    }

    Ok((typetag, typ, args.into_boxed_slice()))
}

fn fmt_typedeforref<'db>(t: &TypeDefOrRef<'db>, f: &mut fmt::Formatter) -> fmt::Result {
    // ECMA-335, II.7.3
    // FIXME: implement correctly and move to impl Debug for TypeRef ...
    match t {
        TypeDefOrRef::TypeDef(_d) => write!(f, "TYPEDEF"), // TODO
        TypeDefOrRef::TypeRef(r) => write!(f, "{}.{}", r.type_namespace().map_err(|_| fmt::Error)?, r.type_name().map_err(|_| fmt::Error)?),
        TypeDefOrRef::TypeSpec(_s) => write!(f, "TYPESPEC"), // TODO
    }
}

impl<'db> fmt::Debug for Type<'db> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // ECMA-335, II.7.1
        use Type::*;
        match *self {
            Primitive(prim) => write!(f, "{:?}", prim),
            Array(ref array) => write!(f, "{:?}[]", array.elem_type()), // TODO: array shape?
            Ref(tag, ref t, ref generic) => {
                write!(f, "{:?} ", tag)?;
                fmt_typedeforref(t, f)?;
                if let Some(g) = generic {
                    write!(f, "<")?;
                    let mut first = true;
                    for arg in g.iter() {
                        if !first { write!(f, ", ")?; }
                        first = false;
                        write!(f, "{:?}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            GenericVar(GenericVarScope::Type, n) => write!(f, "!{}", n),
            GenericVar(GenericVarScope::Method, n) => write!(f, "!!{}", n),
            Object => write!(f, "object"),
            String => write!(f, "string")
        }
    }
}

pub enum RetTypeKind<'db> {
    Void,
    Type(Type<'db>),
    TypeByRef(Type<'db>),
    TypedReference // System.TypedReference
}

impl<'db> fmt::Debug for RetTypeKind<'db> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RetTypeKind::*;
        // ECMA-335, II.7.1
        match *self {
            Void => write!(f, "void")?,
            Type(ref t) => write!(f, "{:?}", t)?,
            TypeByRef(ref t) => write!(f, "{:?}&", t)?,
            TypedReference => write!(f, "typedref")?
        }
        Ok(())
    }
}

// II.23.2.11
pub struct RetType<'db> {
    m_cmod: Vec<CustomMod<'db>>,
    m_kind: RetTypeKind<'db>,
}

impl<'db> RetType<'db> {
    fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<RetType<'db>> {
        let cmod = CustomMod::parse(cur, db)?;

        let mut cur_clone = cur.clone(); // maybe we need to rewind
        let element_type = uncompress_unsigned(cur)?;
        let kind = match element_type as u8 {
            bits::ELEMENT_TYPE_VOID => RetTypeKind::Void,
            bits::ELEMENT_TYPE_BYREF => RetTypeKind::TypeByRef(Type::parse(cur, db)?),
            bits::ELEMENT_TYPE_TYPEDBYREF => RetTypeKind::TypedReference,
            _ => {
                mem::swap(cur, &mut cur_clone); // rewind cursor
                RetTypeKind::Type(Type::parse(cur, db)?)
            }
        };

        Ok(RetType {
            m_cmod: cmod,
            m_kind: kind
        })
    }

    pub fn custom_mod(&self) -> &[CustomMod<'db>] {
        &self.m_cmod[..]
    }

    pub fn kind(&self) -> &RetTypeKind<'db> {
        &self.m_kind
    }
}

pub enum ParamKind<'db> {
    Type(Type<'db>),
    TypeByRef(Type<'db>),
    TypedReference // System.TypedReference
}

impl<'db> fmt::Debug for ParamKind<'db> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParamKind::*;
        match *self {
            // TODO: improve debug printing or remove this
            Type(ref t) => write!(f, "{:?}", t)?,
            TypeByRef(ref t) => write!(f, "byref {:?}", t)?,
            TypedReference => write!(f, "System.TypedReference")?
        }
        Ok(())
    }
}

// II.23.2.10 (renamed to prevent name conflict with Param table row)
pub struct ParamSig<'db> {
    m_cmod: Vec<CustomMod<'db>>,
    m_kind: ParamKind<'db>,
}

impl<'db> ParamSig<'db> {
    fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<ParamSig<'db>> {
        let cmod = CustomMod::parse(cur, db)?;

        let mut cur_clone = cur.clone(); // maybe we need to rewind
        let element_type = uncompress_unsigned(cur)?;
        let kind = match element_type as u8 {
            bits::ELEMENT_TYPE_BYREF => ParamKind::TypeByRef(Type::parse(cur, db)?),
            bits::ELEMENT_TYPE_TYPEDBYREF => ParamKind::TypedReference,
            _ => {
                mem::swap(cur, &mut cur_clone); // rewind cursor
                ParamKind::Type(Type::parse(cur, db)?)
            }
        };

        Ok(ParamSig {
            m_cmod: cmod,
            m_kind: kind
        })
    }

    pub fn custom_mod(&self) -> &[CustomMod<'db>] {
        &self.m_cmod[..]
    }

    pub fn kind(&self) -> &ParamKind<'db> {
        &self.m_kind
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CustomModTag {
    Optional,
    Required
}

// II.23.2.7
pub struct CustomMod<'db> {
    m_tag: CustomModTag,
    m_type: TypeDefOrRef<'db>
}

impl<'db> CustomMod<'db> {
    fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<Vec<CustomMod<'db>>> {
        let mut result = Vec::new();

        loop {
            let mut cur_clone = cur.clone();
            let element_type = uncompress_unsigned(cur)?;
            let tag = match element_type as u8 {
                bits::ELEMENT_TYPE_CMOD_OPT => CustomModTag::Optional,
                bits::ELEMENT_TYPE_CMOD_REQD => CustomModTag::Required,
                _ => {
                    mem::swap(cur, &mut cur_clone); // rewind cursor
                    break
                }
            };
            result.push(CustomMod {
                m_tag: tag,
                m_type: TypeDefOrRef::decode(uncompress_unsigned(cur)?, db)?.expect("Null type in CustomMod")
            });
        }

        Ok(result)
    }

    pub fn tag(&self) -> CustomModTag {
        self.m_tag
    }

    pub fn type_(&self) -> &TypeDefOrRef<'db> {
        &self.m_type
    }
}

// II.23.2.14 (renamed to prevent name clash with TypeSpec table row)
pub enum TypeSpecSig<'db> {
    GenericInst(TypeTag, TypeDefOrRef<'db>, Box<[Type<'db>]>)
}

impl<'db> TypeSpecSig<'db> {
    pub(crate) fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<TypeSpecSig<'db>> {
        let element_type = uncompress_unsigned(cur)?;
        match element_type as u8 {
            bits::ELEMENT_TYPE_PTR | 
            bits::ELEMENT_TYPE_FNPTR |
            bits::ELEMENT_TYPE_ARRAY |
            bits::ELEMENT_TYPE_SZARRAY => unimplemented!(),
            bits::ELEMENT_TYPE_GENERICINST => {
                let (typetag, typ, args) = parse_generic_inst(cur, db)?;
                Ok(TypeSpecSig::GenericInst(typetag, typ, args))
            },
            b => return Err(format!("Unexpected element type for TypeSpec: {}", b).into())
        }
    }
}

impl<'db> fmt::Debug for TypeSpecSig<'db> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // compare impl Debug for Type
        match *self {
            TypeSpecSig::GenericInst(tag, ref t, ref generic) => {
                write!(f, "{:?} ", tag)?;
                fmt_typedeforref(t, f)?;
                write!(f, "<")?;
                let mut first = true;
                for arg in generic.iter() {
                    if !first { write!(f, ", ")?; }
                    first = false;
                    write!(f, "{:?}", arg)?;
                }
                write!(f, ">")
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;

    fn uncompress_unsigned(data: &[u8]) -> crate::Result<u32> {
        let mut cur = Cursor::new(data);
        super::uncompress_unsigned(&mut cur)
    }

    #[test]
    fn test_uncompress_unsigned() {
        assert_eq!(uncompress_unsigned(&[0x03]).unwrap(), 3);
        assert_eq!(uncompress_unsigned(&[0x7F]).unwrap(), 0x7F);
        assert_eq!(uncompress_unsigned(&[0x80, 0x80]).unwrap(), 0x80);
        assert_eq!(uncompress_unsigned(&[0xAE, 0x57]).unwrap(), 0x2E57);
        assert_eq!(uncompress_unsigned(&[0xBF, 0xFF]).unwrap(), 0x3FFF);
        assert_eq!(uncompress_unsigned(&[0xC0, 0x00, 0x40, 0x00]).unwrap(), 0x4000);
        assert_eq!(uncompress_unsigned(&[0xDF, 0xFF, 0xFF, 0xFF]).unwrap(), 0x1FFFFFFF);
    }

    // fn uncompress_signed(data: &[u8]) -> crate::Result<u32> {
    //     let mut cur = Cursor::new(data);
    //     super::uncompress_signed(&mut cur)
    // }

    // #[test]
    // fn test_uncompress_signed() {
    //     assert_eq!(uncompress_signed(&[0x06]).unwrap(), 3);
    //     assert_eq!(uncompress_signed(&[0x7B]).unwrap(), -3);
    //     assert_eq!(uncompress_signed(&[0x80, 0x80]).unwrap(), 64);
    //     assert_eq!(uncompress_signed(&[0x01]).unwrap(), -64);
    //     assert_eq!(uncompress_signed(&[0xC0, 0x00, 0x40, 0x00]).unwrap(), 8192);
    //     assert_eq!(uncompress_signed(&[0x80, 0x01]).unwrap(), -8192);
    //     assert_eq!(uncompress_signed(&[0xDF, 0xFF, 0xFF, 0xFF]).unwrap(), 268435455);
    //     assert_eq!(uncompress_signed(&[0xC0, 0x00, 0x00, 0x01]).unwrap(), -268435456);
    // }
}
