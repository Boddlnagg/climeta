use crate::BitView;

pub struct MethodImplAttributes(pub(crate) u16);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const CodeType_mask: u16 = 0x0003;
    pub const Managed_mask: u16 = 0x0004;
    pub const ForwardRef_bit: usize = 4; // Method is defined; used primarily in merge scenarios
    pub const PreserveSig_bit: usize = 7; // Reserved
    pub const InternalCall_bit: usize = 12; // Reserved
    pub const Synchronized_bit: usize = 5; // Method is single threaded through the body
    pub const NoInlining_bit: usize = 3; // Method cannot be inlined
    pub const NoOptimization_bit: usize = 6; // Method will not be optimized when generatinv native code
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum CodeType {
    IL = 0x0000,      // Method impl is CIL
    Native = 0x0001,  // Method impl is native
    OPTIL = 0x0002,   // Reserved: shall be zero in conforming implementations
    Runtime = 0x0003, // Method impl is provided by the runtime
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum Managed {
    Unmanaged = 0x0004,
    Managed = 0x0000,
}

impl MethodImplAttributes {
    pub fn code_type(&self) -> CodeType {
        self.0.get_enum::<CodeType>(bits::CodeType_mask)
    }

    pub fn managed(&self) -> Managed {
        self.0.get_enum::<Managed>(bits::Managed_mask)
    }
    
    pub fn forward_ref(&self) -> bool {
        self.0.get_bit(bits::ForwardRef_bit)
    }

    pub fn preserve_sig(&self) -> bool {
        self.0.get_bit(bits::PreserveSig_bit)
    }

    pub fn internal_call(&self) -> bool {
        self.0.get_bit(bits::InternalCall_bit)
    }

    pub fn synchronized(&self) -> bool {
        self.0.get_bit(bits::Synchronized_bit)
    }

    pub fn no_inlining(&self) -> bool {
        self.0.get_bit(bits::NoInlining_bit)
    }

    pub fn no_optimization(&self) -> bool {
        self.0.get_bit(bits::NoOptimization_bit)
    }
}