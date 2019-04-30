use crate::BitView;

pub struct PropertyAttributes(pub(crate) u16);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const SpecialName_bit: usize = 9;
    pub const RTSpecialName_bit: usize = 10;
    pub const HasDefault_bit: usize = 12;
}

impl PropertyAttributes {
    pub fn special_name(&self) -> bool {
        self.0.get_bit(bits::SpecialName_bit)
    }

    pub fn rt_special_name(&self) -> bool {
        self.0.get_bit(bits::RTSpecialName_bit)
    }

    pub fn has_default(&self) -> bool {
        self.0.get_bit(bits::HasDefault_bit)
    }
}