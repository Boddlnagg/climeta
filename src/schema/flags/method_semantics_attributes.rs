use crate::core::BitView;

pub struct MethodSemanticsAttributes(pub(crate) u16);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const Setter_bit: usize = 0;
    pub const Getter_bit: usize = 1;
    pub const Other_bit: usize = 2;
    pub const AddOn_bit: usize = 3;
    pub const RemoveOn_bit: usize = 4;
    pub const Fire_bit: usize = 5;
}

impl MethodSemanticsAttributes {
    pub fn setter(&self) -> bool {
        self.0.get_bit(bits::Setter_bit)
    }

    pub fn getter(&self) -> bool {
        self.0.get_bit(bits::Getter_bit)
    }
    
    pub fn other(&self) -> bool {
        self.0.get_bit(bits::Other_bit)
    }
    
    pub fn add_on(&self) -> bool {
        self.0.get_bit(bits::AddOn_bit)
    }
    
    pub fn remove_on(&self) -> bool {
        self.0.get_bit(bits::RemoveOn_bit)
    }
    
    pub fn fire(&self) -> bool {
        self.0.get_bit(bits::Fire_bit)
    }
}