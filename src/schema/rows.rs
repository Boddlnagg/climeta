use num_traits::FromPrimitive;
use byteorder::{ByteOrder, LittleEndian};

use crate::database::{Col0, Col1, Col2, Col3, Col4, Col5};
use crate::table::TableRowIterator;
use crate::{Result, ByteView};
use crate::{database, schema};
use crate::schema::marker;

// TODO: better name
pub trait TableRowWrap {
    type Table;
    type Out;

    fn wrap(table: Self::Table, row: u32) -> Self::Out;
}

macro_rules! row_type {
    ($ty:ident) => {
        pub struct $ty<'db>(pub(crate) database::Row<'db, schema::marker::$ty>);

        impl<'db> schema::TableRow for $ty<'db> {
            type Kind = schema::marker::$ty;
        }
    }
}

row_type!(Assembly);
row_type!(AssemblyOS);
row_type!(AssemblyProcessor);
row_type!(AssemblyRef);
row_type!(AssemblyRefOS);
row_type!(AssemblyRefProcessor);
row_type!(ClassLayout);
row_type!(Constant);
row_type!(CustomAttribute);
row_type!(DeclSecurity);
row_type!(Event);
row_type!(EventMap);
row_type!(ExportedType);
row_type!(Field);
row_type!(FieldLayout);
row_type!(FieldMarshal);
row_type!(FieldRVA);
row_type!(File);
row_type!(GenericParam);
row_type!(GenericParamConstraint);
row_type!(ImplMap);
row_type!(InterfaceImpl);
row_type!(ManifestResource);
row_type!(MemberRef);
row_type!(MethodDef);
row_type!(MethodImpl);
row_type!(MethodSemantics);
row_type!(MethodSpec);
row_type!(Module);
row_type!(ModuleRef);
row_type!(NestedClass);
row_type!(Param);
row_type!(Property);
row_type!(PropertyMap);
row_type!(StandAloneSig);
row_type!(TypeDef);
row_type!(TypeRef);
row_type!(TypeSpec);

impl<'db> TypeDef<'db> {
    pub fn flags(&self) -> Result<super::flags::TypeAttributes> {
        Ok(super::flags::TypeAttributes(self.0.get_value::<Col0, _>()?))
    }

    pub fn type_name(&self) -> Result<&'db str> {
        self.0.get_string::<Col1>()
    }

    pub fn type_namespace(&self) -> Result<&'db str> {
        self.0.get_string::<Col2>()
    }

    pub fn extends(&self) -> Result<Option<super::TypeDefOrRef<'db>>> {
        self.0.get_coded_index::<Col3, super::TypeDefOrRef>()
    }

    pub fn field_list(&self) -> Result<TableRowIterator<'db, marker::Field>> {
        self.0.get_list::<Col4, marker::Field>()
    }

    pub fn method_list(&self) -> Result<TableRowIterator<'db, marker::MethodDef>> {
        self.0.get_list::<Col5, marker::MethodDef>()
    }
}

impl<'db> AssemblyRef<'db> {
    pub fn public_key_or_token(&self) -> Result<&'db [u8]> {
        self.0.get_blob::<Col2>()
    }

    pub fn name(&self) -> Result<&'db str> {
        self.0.get_string::<Col3>()
    }

    pub fn culture(&self) -> Result<&'db str> {
        self.0.get_string::<Col4>()
    }

    pub fn hash_value(&self) -> Result<&'db str> {
        self.0.get_string::<Col5>()
    }
}

impl<'db> Constant<'db> {
    pub fn typ(&self) -> Result<super::ConstantType> {
        <super::ConstantType as FromPrimitive>::from_u16(self.0.get_value::<Col0, _>()?).ok_or_else(|| "Invalid ConstantType".into())
    }

    pub fn parent(&self) -> Result<Option<super::HasConstant<'db>>> {
        self.0.get_coded_index::<Col1, super::HasConstant>()
    }

    pub fn value(&self) -> Result<super::ConstantValue> {
        use super::ConstantType;
        use super::ConstantValue::*;
        let bytes = self.0.get_blob::<Col2>()?;
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

impl<'db> Event<'db> {
    pub fn event_flags(&self) -> Result<super::flags::EventAttributes> {
        Ok(super::flags::EventAttributes(self.0.get_value::<Col0, _>()?))
    }
}

impl<'db> Field<'db> {
    pub fn flags(&self) -> Result<super::flags::FieldAttributes> {
        Ok(super::flags::FieldAttributes(self.0.get_value::<Col0, _>()?))
    }

    pub fn name(&self) -> Result<&'db str> {
        self.0.get_string::<Col1>()
    }
}

impl<'db> GenericParam<'db> {
    pub fn number(&self) -> Result<u16> {
        self.0.get_value::<Col0, _>()
    }

    pub fn flags(&self) -> Result<super::flags::GenericParamAttributes> {
        Ok(super::flags::GenericParamAttributes(self.0.get_value::<Col1, _>()?))
    }

    pub fn owner(&self) -> Result<Option<super::TypeOrMethodDef<'db>>> {
        self.0.get_coded_index::<Col2, super::TypeOrMethodDef>()
    }

    pub fn name(&self) -> Result<&'db str> {
        self.0.get_string::<Col3>()
    }
}

impl<'db> MethodDef<'db> {
    pub fn rva(&self) -> Result<u32> {
        self.0.get_value::<Col0, _>()
    }
    pub fn impl_flags(&self) -> Result<super::flags::MethodImplAttributes> {
        Ok(super::flags::MethodImplAttributes(self.0.get_value::<Col1, _>()?))
    }

    pub fn flags(&self) -> Result<super::flags::MethodAttributes> {
        Ok(super::flags::MethodAttributes(self.0.get_value::<Col2, _>()?))
    }

    pub fn name(&self) -> Result<&'db str> {
        self.0.get_string::<Col3>()
    }

    pub fn param_list(&self) -> Result<TableRowIterator<'db, marker::Param>> {
        self.0.get_list::<Col5, marker::Param>()
    }
}

impl<'db> MethodSemantics<'db> {
    pub fn semantics(&self) -> Result<super::flags::MethodSemanticsAttributes> {
        Ok(super::flags::MethodSemanticsAttributes(self.0.get_value::<Col0, _>()?))
    }

    pub fn method(&self) -> Result<MethodDef<'db>> {
        self.0.get_target_row::<Col1, marker::MethodDef>()
    }

    pub fn association(&self) -> Result<Option<super::HasSemantics<'db>>> {
        self.0.get_coded_index::<Col2, super::HasSemantics>()
    }
}

impl<'db> Param<'db> {
    pub fn flags(&self) -> Result<super::flags::ParamAttributes> {
        Ok(super::flags::ParamAttributes(self.0.get_value::<Col0, _>()?))
    }

    pub fn sequence(&self) -> Result<u16> {
        self.0.get_value::<Col1, u16>()
    }

    pub fn name(&self) -> Result<&'db str> {
        self.0.get_string::<Col2>()
    }
}

impl<'db> Property<'db> {
    pub fn flags(&self) -> Result<super::flags::PropertyAttributes> {
        Ok(super::flags::PropertyAttributes(self.0.get_value::<Col0, _>()?))
    }
}

impl<'db> TypeRef<'db> {
    pub fn resolution_scope(&self) -> Result<Option<super::ResolutionScope<'db>>> {
        self.0.get_coded_index::<Col0, super::ResolutionScope>()
    }

    pub fn type_name(&self) -> Result<&'db str> {
        self.0.get_string::<Col1>()
    }

    pub fn type_namespace(&self) -> Result<&'db str> {
        self.0.get_string::<Col2>()
    }
}
