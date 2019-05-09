use std::mem;
use byteorder::{ReadBytesExt, LittleEndian};
use crate::Result;
use crate::{Cache, ResolveToTypeDef};
use crate::core::db::Database;
use super::{Type, TypeTag, PrimitiveType, MethodDefSig, ParamKind, bits};

fn read_string<'db>(cursor: &mut &'db [u8]) -> Result<Option<&'db str>> {
    let length = super::uncompress_unsigned(cursor)?;
    if length == 0xff { return Ok(None); }
    let (left, mut right) = cursor.split_at(length as usize);
    mem::swap(cursor, &mut right);
    Ok(Some(std::str::from_utf8(left).map_err(|_| crate::DecodeError("Invalid UTF8 in constant value"))?))
}

// ECMA-335, II.23.3 (renamed to prevent name conflict with CustomAttribute table row)
#[derive(Default)]
pub struct CustomAttributeSig<'db> {
    m_fixed: Vec<FixedArg<'db>>,
    m_named: Vec<NamedArg<'db>>,
}

impl<'db> CustomAttributeSig<'db> {
    pub(crate) fn parse<'c: 'db>(cur: &mut &'db [u8], db: &'db Database<'db>, cache: &Cache<'c>, ctor: &MethodDefSig<'db>) -> Result<CustomAttributeSig<'db>> {
        let prolog = cur.read_u16::<LittleEndian>()?;
        if prolog != 0x0001 {
            return Err("CustomAttribute blobs must start with prolog of 0x0001".into());
        }

        let ctor_params = ctor.params();

        let mut fixed_args = Vec::with_capacity(ctor_params.len());

        for param in ctor_params {
            let elem_kind = match param.kind() {
                ParamKind::Type(t) => {
                    ElemKind::from_fixed_arg_type(&t, cache)?
                },
                _ => return Err("unexpected parameter type for FixedArg".into())
            };
            fixed_args.push(FixedArg::parse(cur, db, elem_kind)?);
        }

        let named_args_count = cur.read_u16::<LittleEndian>()?;
        let mut named_args = Vec::with_capacity(named_args_count as usize);

        for _ in 0.. named_args_count {
            named_args.push(NamedArg::parse(cur, db, cache)?);
        }

        Ok(CustomAttributeSig {
            m_fixed: fixed_args,
            m_named: named_args
        })
    }

    pub fn fixed_args(&self) -> &[FixedArg<'db>] {
        &self.m_fixed[..]
    }

    pub fn named_args(&self) -> &[NamedArg<'db>] {
        &self.m_named[..]
    }
}

#[derive(Debug, Clone)]
pub enum FixedArg<'db> {
    Elem(Elem<'db>),
    Array(Vec<Elem<'db>>)
}

impl<'db> FixedArg<'db> {
    fn parse<'c: 'db>(cur: &mut &'db [u8], db: &'db Database, kind: ElemKind<'c>) -> Result<FixedArg<'db>> {
        Ok(match kind {
            ElemKind::Elem(t) => FixedArg::Elem(t.parse_value(cur)?),
            ElemKind::Array(t) => FixedArg::Array(unimplemented!()) // TODO
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NamedArgName<'db> {
    Field(&'db str),
    Property(&'db str)
}

#[derive(Debug, Clone)]
pub struct NamedArg<'db> {
    pub name: NamedArgName<'db>,
    pub value: FixedArg<'db>
}

impl<'db> NamedArg<'db> {
    fn parse<'c: 'db>(cur: &mut &'db [u8], db: &'db Database, cache: &Cache<'c>) -> Result<NamedArg<'db>> {
        let is_property = match cur.read_u8()? {
            bits::ARG_FIELD => false,
            bits::ARG_PROPERTY => true,
            _ => {
                return Err("NamedArg must be either FIELD or PROPERTY".into());
            }
        };

        let elem_kind = ElemKind::parse(cur, db, cache)?;
        let name = read_string(cur)?.expect("NamedArg name must not be NULL");
        let value = FixedArg::parse(cur, db, elem_kind)?;

        if is_property {
            Ok(NamedArg { name: NamedArgName::Property(name), value: value })
        } else {
            Ok(NamedArg { name: NamedArgName::Field(name), value: value })
        }
    }
}

fn enum_get_underlying_type(typ: &super::TypeDef) -> Result<PrimitiveType> {
    use PrimitiveType::*;

    debug_assert!(typ.is_enum());
    let mut result = None;
    for field in typ.field_list()? {
        let flags = field.flags()?;
        if !flags.literal() && !flags.static_() {
            debug_assert!(result.is_none());
            let typ = match field.signature()?.type_() {
                Type::Primitive(p) => *p,
                _ => return Err("enum underlying type must be primitive".into())
            };
            assert!(match typ { Boolean | Char | I1 | U1 | I2 | U2 | I4 | U4 | I8 | U8 => true, _ => false });
            result = Some(typ);
        }
    }
    Ok(result.expect("enum without underlying type"))
}

enum FieldOrPropType<'db> {
    Primitive(PrimitiveType),
    String,
    SystemType,
    Enum(super::TypeDef<'db>),
}

impl<'db> FieldOrPropType<'db> {
    fn from_fixed_arg_type<'c: 'db>(typ: &Type<'db>, cache: &Cache<'c>) -> Result<FieldOrPropType<'db>> {
        Ok(match typ {
            Type::Primitive(PrimitiveType::I) | Type::Primitive(PrimitiveType::U) => return Err("FieldOrPropType can not have type I or U".into()),
            Type::Primitive(p) => FieldOrPropType::Primitive(*p),
            Type::Ref(_, t, None) if t.namespace_name_pair() == ("System", "Type") => FieldOrPropType::SystemType,
            Type::Ref(TypeTag::ValueType, t, None) => {
                let resolved = t.resolve(cache).ok_or::<crate::DecodeError>("Unresolvable CustomAttribute param TypeDefOrRef".into())?;
                if !resolved.is_enum() {
                    return Err("CustomAttribute params that are TypeDefOrRef must be an enum or System.Type".into())
                }
                FieldOrPropType::Enum(resolved.clone())
            },
            Type::String => FieldOrPropType::String,
            _ => {
                unimplemented!() // TODO: System.Object (boxed value type) is also possible according to II.23.3 §Elem
            }
        })
    }

    fn parse<'c: 'db>(cur: &mut &'db [u8], db: &'db Database<'db>, cache: &Cache<'c>) -> Result<FieldOrPropType<'db>> {
        Ok(match cur.read_u8()? {
            bits::ELEMENT_TYPE_BOOLEAN => FieldOrPropType::Primitive(PrimitiveType::Boolean),
            bits::ELEMENT_TYPE_CHAR => FieldOrPropType::Primitive(PrimitiveType::Char),
            bits::ELEMENT_TYPE_I1 => FieldOrPropType::Primitive(PrimitiveType::I1),
            bits::ELEMENT_TYPE_U1 => FieldOrPropType::Primitive(PrimitiveType::U1),
            bits::ELEMENT_TYPE_I2 => FieldOrPropType::Primitive(PrimitiveType::I2),
            bits::ELEMENT_TYPE_U2 => FieldOrPropType::Primitive(PrimitiveType::U2),
            bits::ELEMENT_TYPE_I4 => FieldOrPropType::Primitive(PrimitiveType::I4),
            bits::ELEMENT_TYPE_U4 => FieldOrPropType::Primitive(PrimitiveType::U4),
            bits::ELEMENT_TYPE_I8 => FieldOrPropType::Primitive(PrimitiveType::I8),
            bits::ELEMENT_TYPE_U8 => FieldOrPropType::Primitive(PrimitiveType::U8),
            bits::ELEMENT_TYPE_R4 => FieldOrPropType::Primitive(PrimitiveType::R4),
            bits::ELEMENT_TYPE_R8 => FieldOrPropType::Primitive(PrimitiveType::R8),
            bits::ELEMENT_TYPE_STRING => FieldOrPropType::String,
            bits::ARG_SYSTEM_TYPE => FieldOrPropType::SystemType,
            bits::ARG_ENUM => {
                let type_string = read_string(cur)?.expect("NamedArg enum type name must not be NULL");
                let type_def = match type_string.resolve(cache) {
                    None => return Err("CustomAttribute named param referenced unresolved enum type".into()),
                    Some(t) => if !t.is_enum() { return Err("CustomAttribute named param referenced non-enum type".into()); } else { t }
                };
                FieldOrPropType::Enum(type_def)
            },
            _ => return Err("unexpected FieldOrPropType".into())
        })
    }

    fn parse_value(self, cur: &mut &'db [u8]) -> Result<Elem<'db>> {
        use FieldOrPropType::*;
        Ok(match self {
            Primitive(p) => Elem::Primitive(p.parse_value(cur)?),
            String => Elem::String(read_string(cur)?),
            SystemType => Elem::SystemType(read_string(cur)?.expect("NULL string in System.Type custom attribute value")),
            Enum(t) => {
                let underlying = enum_get_underlying_type(&t)?;
                Elem::EnumValue(t, underlying.parse_value(cur)?)
            }
        })
    }
}

enum ElemKind<'db> {
    Elem(FieldOrPropType<'db>),
    Array(FieldOrPropType<'db>)
}

impl<'db> ElemKind<'db> {
    fn from_fixed_arg_type<'c: 'db>(typ: &Type<'db>, cache: &Cache<'c>) -> Result<ElemKind<'db>> {
        Ok(match typ {
            Type::Array(array) => ElemKind::Array(FieldOrPropType::from_fixed_arg_type(array.elem_type(), cache)?),
            _ => ElemKind::Elem(FieldOrPropType::from_fixed_arg_type(typ, cache)?)
        })
    }

    fn parse<'c: 'db>(cur: &mut &'db [u8], db: &'db Database<'db>, cache: &Cache<'c>) -> Result<ElemKind<'db>> {
        let mut cur_clone = cur.clone(); // maybe we need to rewind
        let element_type = cur.read_u8()?;
        Ok(match element_type {
            bits::ELEMENT_TYPE_SZARRAY => ElemKind::Array(FieldOrPropType::parse(cur, db, cache)?),
            _ => {
                mem::swap(cur, &mut cur_clone); // rewind cursor
                ElemKind::Elem(FieldOrPropType::parse(cur, db, cache)?)
            }
        })
    }
}

#[derive(Clone, Debug)]
pub enum Elem<'db> {
    Primitive(super::PrimitiveValue),
    String(Option<&'db str>),
    SystemType(&'db str),
    EnumValue(super::TypeDef<'db>, super::PrimitiveValue)
}
