mod assembly_attributes;
mod event_attributes;
mod field_attributes;
mod generic_param_attributes;
mod method_attributes;
mod method_impl_attributes;
mod method_semantics_attributes;
mod param_attributes;
mod property_attributes;
mod type_attributes;

pub use assembly_attributes::*;
pub use event_attributes::*;
pub use field_attributes::*;
pub use generic_param_attributes::*;
pub use method_attributes::*;
pub use method_impl_attributes::*;
pub use method_semantics_attributes::*;
pub use param_attributes::*;
pub use property_attributes::*;
pub use type_attributes::*;

// Shared definitions

#[repr(u32)]
#[derive(FromPrimitive, ToPrimitive)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MemberAccess {
    CompilerControlled = 0x0000, // Member not referenceable
    Private = 0x0001,
    FamAndAssem = 0x0002,        // Accessible by subtypes only in this Assembly
    Assembly = 0x0003,           // Accessible by anyone in this Assembly
    Family = 0x0004,             // aka Protected
    FamOrAssem = 0x0005,         // Accessible by subtypes anywhere, plus anyone in this Assembly
    Public = 0x0006,
}