use crate::core::BitView;

use super::MemberAccess;

pub struct MethodAttributes(pub(crate) u16);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const Access_mask: u16 = 0x0007;
    pub const Static_bit: usize = 4;
    pub const Final_bit: usize = 5;
    pub const Virtual_bit: usize = 6;
    pub const HideBySig_bit: usize = 7;
    pub const VtableLayout_mask: u16 = 0x0100;
    pub const Strict_bit: usize = 9;
    pub const Abstract_bit: usize = 10;
    pub const SpecialName_bit: usize = 11;
    pub const PInvokeImpl_bit: usize = 13;
    pub const UnmanagedExport_bit: usize = 3;
    pub const RTSpecialName_bit: usize = 12;
    pub const HasSecurity_bit: usize = 14;
    pub const RequireSecObject_bit: usize = 15;
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum VtableLayout {
    ReuseSlot = 0x0000, // Method reuses existing slot in a vtable
    NewSlot = 0x0100,   // Method always gets a new slot in the vtable
}

impl MethodAttributes {
    pub fn access(&self) -> MemberAccess {
        self.0.get_enum::<MemberAccess>(bits::Access_mask)
    }
    pub fn static_(&self) -> bool {
        self.0.get_bit(bits::Static_bit)
    }
    pub fn final_(&self) -> bool {
        self.0.get_bit(bits::Final_bit)
    }
    pub fn virtual_(&self) -> bool {
        self.0.get_bit(bits::Virtual_bit)
    }
    pub fn hide_by_sig(&self) -> bool {
        self.0.get_bit(bits::HideBySig_bit)
    }
    pub fn layout(&self) -> VtableLayout {
        self.0.get_enum::<VtableLayout>(bits::VtableLayout_mask)
    }
    pub fn strict(&self) -> bool {
        self.0.get_bit(bits::Strict_bit)
    }
    pub fn abstract_(&self) -> bool {
        self.0.get_bit(bits::Abstract_bit)
    }
    pub fn special_name(&self) -> bool {
        self.0.get_bit(bits::SpecialName_bit)
    }
    pub fn pinvoke_impl(&self) -> bool {
        self.0.get_bit(bits::PInvokeImpl_bit)
    }
    pub fn unmanaged_export(&self) -> bool {
        self.0.get_bit(bits::UnmanagedExport_bit)
    }
    pub fn rt_special_name(&self) -> bool {
        self.0.get_bit(bits::RTSpecialName_bit)
    }
    pub fn has_security(&self) -> bool {
        self.0.get_bit(bits::HasSecurity_bit)
    }
    pub fn require_sec_object(&self) -> bool {
        self.0.get_bit(bits::RequireSecObject_bit)
    }
}