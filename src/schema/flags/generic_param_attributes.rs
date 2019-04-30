use crate::BitView;

pub struct GenericParamAttributes(pub(crate) u16);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const Variance_mask: u16 = 0x0003;
    pub const SpecialConstraint_mask: u16 = 0x001c;
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum GenericParamVariance {
    None = 0x0000,
    Covariant = 0x0001,
    Contravariant = 0x0002
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum GenericParamSpecialConstraint {
    ReferenceTypeConstraint = 0x0004,
    NotNullableValueTypeConstraint = 0x0008,
    DefaultConstructorConstraint = 0x0010
}

impl GenericParamAttributes {
    pub fn variance(&self) -> GenericParamVariance {
        self.0.get_enum::<GenericParamVariance>(bits::Variance_mask)
    }

    pub fn special_constraint(&self) -> GenericParamSpecialConstraint {
        self.0.get_enum::<GenericParamSpecialConstraint>(bits::SpecialConstraint_mask)
    }
}