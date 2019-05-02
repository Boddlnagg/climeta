use std::io::Cursor;
use std::fmt;
use byteorder::ReadBytesExt;
use crate::Result;
use crate::database::{Database, CodedIndex};
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
        //length = 1;
        Ok(first as u32)
    } else if (first & 0xc0) == 0x80 {
        //length = 2;
        let mut value = ((first & 0x3f) as u32) << 8;
        value |= cursor.read_u8()? as u32;
        Ok(value)
    } else if (first & 0xe0) == 0xc0 {
        //length = 4;
        let mut value = ((first & 0x1f) as u32) << 24;
        value |= (cursor.read_u8()? as u32) << 16;
        value |= (cursor.read_u8()? as u32) << 8;
        value |= cursor.read_u8()? as u32;
        Ok(value)
    } else {
        Err("Invalid compressed integer in blob".into())
    }
}

/*fn uncompress_enum<T: FromPrimitive>(cursor: &mut Cursor<&[u8]>) -> Result<T> {
    T::from_u32(uncompress_unsigned(cursor)?)
}*/

/*#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum CallingConvention {
    Default = 0x00,
    VarArg = 0x05,
    Field = 0x06,
    LocalSig = 0x07,
    Property = 0x08,
    GenericInst = 0x10,
    Mask = 0x0f,

    HasThis = 0x20,
    ExplicitThis = 0x40,
    Generic = 0x10,
}*/

#[allow(non_upper_case_globals)]
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

// for MethodDefSig
#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum CallingConvention {
    Default = bits::DEFAULT,
    VarArg = bits::VARARG, // not allowed for CLS
    Generic = bits::GENERIC
}


pub struct MethodDefSig<'db> {
    m_initial_byte: u8,
    m_generic_param_count: u32,
    m_param_count: u32,
    m_ret_type: RetTypeSig<'db>,
    m_params: Box<[ParamSig<'db>]>, // TODO: iterator?
}

impl<'db> MethodDefSig<'db> {
    // TODO: probably should make this consistent with the other `parse` functions
    pub(crate) fn decode(data: &'db [u8], db: &'db Database) -> Result<MethodDefSig<'db>> {
        let mut cursor = Cursor::new(data);
        let cur = &mut cursor;
        let initial_byte = cur.read_u8()?;
        let generic_param_count = if initial_byte & bits::GENERIC != 0 {
            uncompress_unsigned(cur)?
        } else {
            0
        };

        let param_count = uncompress_unsigned(cur)?;

        let ret_type = RetTypeSig::parse(cur, db)?;

        let mut params = Vec::with_capacity(param_count as usize);
        
        for _ in 0..param_count {
            params.push(ParamSig::parse(cur, db)?);
        }

        Ok( MethodDefSig {
            m_initial_byte: initial_byte,
            m_generic_param_count: generic_param_count,
            m_param_count: param_count,
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

    pub fn calling_convention(&self) -> CallingConvention {
        // FIXME
        //CallingConvention self.m_initial_byte & bits::CallingConvention_mask
        unimplemented!()
    }

    pub fn generic_param_count(&self) -> u32 {
        self.m_generic_param_count
    }

    pub fn return_type(&self) -> &RetTypeSig<'db> {
        &self.m_ret_type
    }
    
    pub fn params(&self) -> &[ParamSig<'db>] {
        &self.m_params
    }
}

// TODO: this could also internally be Box<(TypeSig, [CustomModSig])>,
//       where the tuple is dynamically sized, to have only one dynamic allocation
pub struct Array<'db> {
    m_type: Box<TypeSig<'db>>,
    m_cmod: Vec<CustomModSig<'db>>
    // TODO: optional ArrayShape
}

impl<'db> Array<'db> {
    fn parse_szarray(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<Array<'db>> {
        // ELEMENT_TYPE_SZARRAY already consumed
        let cmod = CustomModSig::parse(cur, db)?;
        Ok(Array {
            m_type: Box::new(TypeSig::parse(cur, db)?),
            m_cmod: cmod
        })
    }

    pub fn elem_type(&self) -> &TypeSig<'db> {
        &self.m_type
    }

    pub fn custom_mod(&self) -> &[CustomModSig<'db>] {
        &self.m_cmod[..]
    }
}


impl<'db> fmt::Debug for Array<'db> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: improve debug printing or remove this
        write!(f, "{:?}[]", self.m_type)
    }
}



#[derive(Debug)]
pub enum TypeSig<'db> {
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
    Array(Array<'db>), // for ARRAY and SZARRAY
    Class(TypeDefOrRef<'db>),
    FnPtr, // TODO
    GenericClassInst(TypeDefOrRef<'db>, Box<[TypeSig<'db>]>),
    GenericValueTypeInst(TypeDefOrRef<'db>, Box<[TypeSig<'db>]>),
    MVar(u32),
    Object,
    Ptr, // TODO
    String,
    ValueType(TypeDefOrRef<'db>),
    Var(u32),
}

impl<'db> TypeSig<'db> {
    fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<TypeSig<'db>> {
        let element_type = uncompress_unsigned(cur)?;
        Ok(match element_type as u8 {
            bits::ELEMENT_TYPE_BOOLEAN => TypeSig::Boolean,
            bits::ELEMENT_TYPE_CHAR => TypeSig::Char,
            bits::ELEMENT_TYPE_I1 => TypeSig::I1,
            bits::ELEMENT_TYPE_U1 => TypeSig::U1,
            bits::ELEMENT_TYPE_I2 => TypeSig::I2,
            bits::ELEMENT_TYPE_U2 => TypeSig::U2,
            bits::ELEMENT_TYPE_I4 => TypeSig::I4,
            bits::ELEMENT_TYPE_U4 => TypeSig::U4,
            bits::ELEMENT_TYPE_I8 => TypeSig::I8,
            bits::ELEMENT_TYPE_U8 => TypeSig::U8,
            bits::ELEMENT_TYPE_R4 => TypeSig::R4,
            bits::ELEMENT_TYPE_R8 => TypeSig::R8,
            bits::ELEMENT_TYPE_I => TypeSig::I,
            bits::ELEMENT_TYPE_U => TypeSig::U,
            bits::ELEMENT_TYPE_ARRAY => unimplemented!(),
            bits::ELEMENT_TYPE_CLASS => TypeSig::Class(TypeDefOrRef::decode(uncompress_unsigned(cur)?, db)?.expect("Null type in Class TypeSig")),
            bits::ELEMENT_TYPE_FNPTR => unimplemented!(),
            bits::ELEMENT_TYPE_GENERICINST => Self::parse_generic_inst(cur, db)?,
            bits::ELEMENT_TYPE_MVAR => TypeSig::MVar(uncompress_unsigned(cur)?),
            bits::ELEMENT_TYPE_OBJECT => TypeSig::Object,
            bits::ELEMENT_TYPE_PTR => unimplemented!(),
            bits::ELEMENT_TYPE_STRING => TypeSig::String,
            bits::ELEMENT_TYPE_SZARRAY => TypeSig::Array(Array::parse_szarray(cur, db)?),
            bits::ELEMENT_TYPE_VALUETYPE => TypeSig::ValueType(TypeDefOrRef::decode(uncompress_unsigned(cur)?, db)?.expect("Null type in ValueType TypeSig")),
            bits::ELEMENT_TYPE_VAR => TypeSig::Var(uncompress_unsigned(cur)?),
            b => return Err(format!("Unexpected element type for TypeSig: {}", b).into())
        })
    }

    fn parse_generic_inst(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<TypeSig<'db>> {
        let is_valuetype = match uncompress_unsigned(cur)? as u8 {
            bits::ELEMENT_TYPE_CLASS => false,
            bits::ELEMENT_TYPE_VALUETYPE => true,
            _ => return Err("Generic type instantiation signatures must begin with either ELEMENT_TYPE_CLASS or ELEMENT_TYPE_VALUE".into())
        };

        let typ = TypeDefOrRef::decode(uncompress_unsigned(cur)?, db)?.expect("Null type in GenericInst arg");
        let arg_count = uncompress_unsigned(cur)?;
        let mut args = Vec::with_capacity(arg_count as usize);
        for _ in 0..arg_count {
            args.push(TypeSig::parse(cur, db)?);
        }
        if is_valuetype {
            Ok(TypeSig::GenericClassInst(typ, args.into_boxed_slice()))
        } else {
            Ok(TypeSig::GenericValueTypeInst(typ, args.into_boxed_slice()))
        }
    }
}

pub enum RetTypeKind<'db> {
    Void,
    Type(TypeSig<'db>),
    TypeByRef(TypeSig<'db>),
    TypedReference // System.TypedReference
}

impl<'db> fmt::Debug for RetTypeKind<'db> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RetTypeKind::*;
        match *self {
            // TODO: improve debug printing or remove this
            Void => write!(f, "void")?,
            Type(ref t) => write!(f, "{:?}", t)?,
            TypeByRef(ref t) => write!(f, "byref {:?}", t)?,
            TypedReference => write!(f, "System.TypedReference")?
        }
        Ok(())
    }
}

pub struct RetTypeSig<'db> {
    m_cmod: Vec<CustomModSig<'db>>,
    m_kind: RetTypeKind<'db>,
}

impl<'db> RetTypeSig<'db> {
    fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<RetTypeSig<'db>> {
        let cmod = CustomModSig::parse(cur, db)?;

        let mut cur_clone = cur.clone(); // maybe we need to rewind
        let element_type = uncompress_unsigned(cur);
        let kind = match element_type? as u8 {
            bits::ELEMENT_TYPE_VOID => RetTypeKind::Void,
            bits::ELEMENT_TYPE_BYREF => RetTypeKind::TypeByRef(TypeSig::parse(cur, db)?),
            bits::ELEMENT_TYPE_TYPEDBYREF => RetTypeKind::TypedReference,
            _ => {
                std::mem::swap(cur, &mut cur_clone); // rewind cursor
                RetTypeKind::Type(TypeSig::parse(cur, db)?)
            }
        };

        Ok(RetTypeSig {
            m_cmod: cmod,
            m_kind: kind
        })
    }

    pub fn custom_mod(&self) -> &[CustomModSig<'db>] {
        &self.m_cmod[..]
    }

    pub fn kind(&self) -> &RetTypeKind<'db> {
        &self.m_kind
    }
}

pub enum ParamKind<'db> {
    Type(TypeSig<'db>),
    TypeByRef(TypeSig<'db>),
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

pub struct ParamSig<'db> {
    m_cmod: Vec<CustomModSig<'db>>,
    m_kind: ParamKind<'db>,
}

impl<'db> ParamSig<'db> {
    fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<ParamSig<'db>> {
        let cmod = CustomModSig::parse(cur, db)?;

        let mut cur_clone = cur.clone(); // maybe we need to rewind
        let element_type = uncompress_unsigned(cur)?;
        let kind = match element_type as u8 {
            bits::ELEMENT_TYPE_BYREF => ParamKind::TypeByRef(TypeSig::parse(cur, db)?),
            bits::ELEMENT_TYPE_TYPEDBYREF => ParamKind::TypedReference,
            _ => {
                std::mem::swap(cur, &mut cur_clone); // rewind cursor
                ParamKind::Type(TypeSig::parse(cur, db)?)
            }
        };

        Ok(ParamSig {
            m_cmod: cmod,
            m_kind: kind
        })
    }

    pub fn custom_mod(&self) -> &[CustomModSig<'db>] {
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

pub struct CustomModSig<'db> {
    m_tag: CustomModTag,
    m_type: TypeDefOrRef<'db>
}

impl<'db> CustomModSig<'db> {
    fn parse(cur: &mut Cursor<&'db [u8]>, db: &'db Database) -> Result<Vec<CustomModSig<'db>>> {
        let mut result = Vec::new();

        loop {
            let mut cur_clone = cur.clone();
            let element_type = uncompress_unsigned(cur);
            let tag = match element_type? as u8 {
                bits::ELEMENT_TYPE_CMOD_OPT => CustomModTag::Optional,
                bits::ELEMENT_TYPE_CMOD_REQD => CustomModTag::Required,
                _ => {
                    std::mem::swap(cur, &mut cur_clone); // rewind cursor
                    break
                }
            };
            result.push(CustomModSig {
                m_tag: tag,
                m_type: TypeDefOrRef::decode(uncompress_unsigned(cur)?, db)?.expect("Null type in CustomModSig")
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
