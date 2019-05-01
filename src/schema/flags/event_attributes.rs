use crate::core::BitView;

pub struct EventAttributes(pub(crate) u16);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const SpecialName_bit: usize = 9;
    pub const RTSpecialName_bit: usize = 10;
}

impl EventAttributes {
    pub fn special_name(&self) -> bool {
        self.0.get_bit(bits::SpecialName_bit)
    }

    pub fn rt_special_name(&self) -> bool {
        self.0.get_bit(bits::RTSpecialName_bit)
    }
}