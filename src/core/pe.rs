#![allow(non_snake_case)]

#[repr(C)]
pub struct image_dos_header
{
    pub e_magic: u16,
    pub e_cblp: u16,
    pub e_cp: u16,
    pub e_crlc: u16,
    pub e_cparhdr: u16,
    pub e_minalloc: u16,
    pub e_maxalloc: u16,
    pub e_ss: u16,
    pub e_sp: u16,
    pub e_csum: u16,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_lfarlc: u16,
    pub e_ovno: u16,
    pub e_res: [u16; 4],
    pub e_oemid: u16,
    pub e_oeminfo: u16,
    pub e_res2: [u16; 10],
    pub e_lfanew: i32,
}

#[repr(C)]
pub struct image_file_header
{
    pub Machine: u16,
    pub NumberOfSections: u16,
    pub TimeDateStamp: u32,
    pub PointerToSymbolTable: u32,
    pub NumberOfSymbols: u32,
    pub SizeOfOptionalHeader: u16,
    pub Characteristics: u16,
}

#[repr(C)]
pub struct image_data_directory
{
    pub VirtualAddress: u32,
    pub Size: u32,
}

#[repr(C)]
pub struct image_optional_header32
{
    pub Magic: u16,
    pub MajorLinkerVersion: u8,
    pub MinorLinkerVersion: u8,
    pub SizeOfCode: u32,
    pub SizeOfInitializedData: u32,
    pub SizeOfUninitializedData: u32,
    pub AddressOfEntryPoint: u32,
    pub BaseOfCode: u32,
    pub BaseOfData: u32,
    pub ImageBase: u32,
    pub SectionAlignment: u32,
    pub FileAlignment: u32,
    pub MajorOperatingSystemVersion: u16,
    pub MinorOperatingSystemVersion: u16,
    pub MajorImageVersion: u16,
    pub MinorImageVersion: u16,
    pub MajorSubsystemVersion: u16,
    pub MinorSubsystemVersion: u16,
    pub Win32VersionValue: u32,
    pub SizeOfImage: u32,
    pub SizeOfHeaders: u32,
    pub CheckSum: u32,
    pub Subsystem: u16,
    pub DllCharacteristics: u16,
    pub SizeOfStackReserve: u32,
    pub SizeOfStackCommit: u32,
    pub SizeOfHeapReserve: u32,
    pub SizeOfHeapCommit: u32,
    pub LoaderFlags: u32,
    pub NumberOfRvaAndSizes: u32,
    pub DataDirectory: [image_data_directory; 16],
}

#[repr(C)]
pub struct image_nt_headers32
{
    pub Signature: u32,
    pub FileHeader: image_file_header,
    pub OptionalHeader: image_optional_header32,
}

#[repr(C)]
pub struct image_section_header {
    pub Name: [u8; 8], // IMAGE_SIZEOF_SHORT_NAME
    pub Union_PhysicalAddress_VirtualSize: u32,
    pub VirtualAddress: u32,
    pub SizeOfRawData: u32,
    pub PointerToRawData: u32,
    pub PointerToRelocations: u32,
    pub PointerToLinenumbers: u32,
    pub NumberOfRelocations: u16,
    pub NumberOfLinenumbers: u16,
    pub Characteristics: u32,
}

#[repr(C)]
pub struct image_cor20_header
{
    pub cb: u32,
    pub MajorRuntimeVersion: u16,
    pub MinorRuntimeVersion: u16,
    pub MetaData: image_data_directory,
    pub Flags: u32,
    pub Union_EntryPointToken_EntryPointRVA: u32,
    pub Resources: image_data_directory,
    pub StrongNameSignature: image_data_directory,
    pub CodeManagerTable: image_data_directory,
    pub VTableFixups: image_data_directory,
    pub ExportAddressTableJumps: image_data_directory,
    pub ManagedNativeHeader: image_data_directory,
}

pub fn section_from_rva(sections: &[image_section_header], rva: u32) -> Option<&image_section_header> {
    sections.iter().find(|section| rva >= section.VirtualAddress && rva < section.VirtualAddress + section.Union_PhysicalAddress_VirtualSize)
}

pub fn offset_from_rva(section: &image_section_header, rva: u32) -> usize {
    (rva - section.VirtualAddress + section.PointerToRawData) as usize
}
