use crate::core::BitView;

pub struct ParamAttributes(pub(crate) u16);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const In_bit: usize = 0;
    pub const Out_bit: usize = 1;
    pub const Optional_bit: usize = 4;
    pub const HasDefault_bit: usize = 12;
    pub const HasFieldMarshal_bit: usize = 13;
}

impl ParamAttributes {
    pub fn in_(&self) -> bool {
        self.0.get_bit(bits::In_bit)
    }

    pub fn out(&self) -> bool {
        self.0.get_bit(bits::Out_bit)
    }

    pub fn optional(&self) -> bool {
        self.0.get_bit(bits::Optional_bit)
    }

    pub fn has_default(&self) -> bool {
        self.0.get_bit(bits::HasDefault_bit)
    }

    pub fn has_field_marshal(&self) -> bool {
        self.0.get_bit(bits::HasFieldMarshal_bit)
    }
}