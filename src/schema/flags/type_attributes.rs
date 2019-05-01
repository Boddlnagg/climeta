use crate::core::BitView;

pub struct TypeAttributes(pub(crate) u32);

#[allow(non_upper_case_globals)]
pub(super) mod bits {
    pub const Visibility_mask: u32 = 0x00000007;
    pub const Layout_mask: u32 = 0x00000018;
    pub const Semantics_mask: u32 = 0x00000020;
    pub const Abstract_bit: usize = 7;
    pub const Sealed_bit: usize = 8;
    pub const SpecialName_bit: usize = 10;
    pub const Import_bit: usize = 12;
    pub const Serializable_bit: usize = 13;
    pub const WindowsRuntime_bit: usize = 14;
    pub const StringFormat_mask: u32 = 0x00030000;
    pub const BeforeFieldInit_bit: usize = 20;
    pub const RTSpecialName_bit: usize = 11;
    pub const HasSecurity_bit: usize = 18;
    pub const IsTypeForwarder_bit: usize = 21;
}

#[repr(u32)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum TypeVisibility {
    NotPublic = 0x00000000,
    Public = 0x00000001,
    NestedPublic = 0x00000002,
    NestedPrivate = 0x00000003,
    NestedFamily = 0x00000004,
    NestedAssembly = 0x00000005,
    NestedFamANDAssem = 0x00000006,
    NestedFamORAssem = 0x00000007,
}

#[repr(u32)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum TypeLayout {
    AutoLayout = 0x00000000,
    SequentialLayout = 0x00000008,
    ExplicitLayout = 0x00000010,
}

#[repr(u32)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum TypeSemantics {
    Class = 0x00000000,
    Interface = 0x00000020,
}

#[repr(u32)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug)]
pub enum StringFormat {
    AnsiClass = 0x00000000,
    UnicodeClass = 0x00010000,
    AutoClass = 0x00020000,
    CustomFormatClass = 0x00030000,
    CustomFormatMask = 0x00C00000,
}

impl TypeAttributes {
    pub fn visibility(&self) -> TypeVisibility {
        self.0.get_enum::<TypeVisibility>(bits::Visibility_mask)
    }

    pub fn layout(&self) -> TypeLayout {
        self.0.get_enum::<TypeLayout>(bits::Layout_mask)
    }

    pub fn semantics(&self) -> TypeSemantics {
        self.0.get_enum::<TypeSemantics>(bits::Semantics_mask)
    }

    pub fn abstract_(&self) -> bool {
        self.0.get_bit(bits::Abstract_bit)
    }

    pub fn sealed(&self) -> bool {
        self.0.get_bit(bits::Sealed_bit)
    }

    pub fn special_name(&self) -> bool {
        self.0.get_bit(bits::SpecialName_bit)
    }

    pub fn import(&self) -> bool {
        self.0.get_bit(bits::Import_bit)
    }

    pub fn serializable(&self) -> bool {
        self.0.get_bit(bits::Serializable_bit)
    }

    pub fn windows_runtime(&self) -> bool {
        self.0.get_bit(bits::WindowsRuntime_bit)
    }

    pub fn string_format(&self) -> StringFormat {
        self.0.get_enum::<StringFormat>(bits::StringFormat_mask)
    }

    pub fn before_field_init(&self) -> bool {
        self.0.get_bit(bits::BeforeFieldInit_bit)
    }

    pub fn rt_special_name(&self) -> bool {
        self.0.get_bit(bits::RTSpecialName_bit)
    }

    pub fn has_security(&self) -> bool {
        self.0.get_bit(bits::HasSecurity_bit)
    }

    pub fn is_type_forwarder(&self) -> bool {
        self.0.get_bit(bits::IsTypeForwarder_bit)
    }
}

