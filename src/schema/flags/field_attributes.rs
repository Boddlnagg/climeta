use crate::core::BitView;

use super::MemberAccess;

pub struct FieldAttributes(pub(crate) u16);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const Access_mask: u16 = 0x0007;
    pub const Static_bit: usize = 4;
    pub const InitOnly_bit: usize = 5;
    pub const Literal_bit: usize = 6;
    pub const NotSerialized_bit: usize = 7;
    pub const SpecialName_bit: usize = 9;
    pub const PInvokeImpl_bit: usize = 13;
    pub const RTSpecialName_bit: usize = 10;
    pub const HasFieldMarshal_bit: usize = 12;
    pub const HasDefault_bit: usize = 15;
    pub const HasFieldRVA_bit: usize = 8;
}

impl FieldAttributes {
    pub fn access(&self) -> MemberAccess {
        self.0.get_enum::<MemberAccess>(bits::Access_mask)
    }

    pub fn static_(&self) -> bool {
        self.0.get_bit(bits::Static_bit)
    }

    pub fn init_only(&self) -> bool {
        self.0.get_bit(bits::InitOnly_bit)
    }

    pub fn literal(&self) -> bool {
        self.0.get_bit(bits::Literal_bit)
    }

    pub fn not_serialized(&self) -> bool {
        self.0.get_bit(bits::NotSerialized_bit)
    }

    pub fn special_name(&self) -> bool {
        self.0.get_bit(bits::SpecialName_bit)
    }

    pub fn pinvoke_impl(&self) -> bool {
        self.0.get_bit(bits::PInvokeImpl_bit)
    }

    pub fn rt_special_name(&self) -> bool {
        self.0.get_bit(bits::RTSpecialName_bit)
    }

    pub fn has_field_marshal(&self) -> bool {
        self.0.get_bit(bits::HasFieldMarshal_bit)
    }

    pub fn has_default(&self) -> bool {
        self.0.get_bit(bits::HasDefault_bit)
    }

    pub fn has_field_rva(&self) -> bool {
        self.0.get_bit(bits::HasFieldRVA_bit)
    }
}