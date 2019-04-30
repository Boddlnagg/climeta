use crate::BitView;

pub struct AssemblyAttributes(pub(crate) u16);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const WindowsRuntime_bit: usize = 9;
}

impl AssemblyAttributes {
    pub fn windows_runtime(&self) -> bool {
        self.0.get_bit(bits::WindowsRuntime_bit)
    }
}