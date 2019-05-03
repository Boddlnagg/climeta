use byteorder::{ByteOrder, LittleEndian};

use super::db::TableDesc;

pub(crate) trait ColumnIndex { fn idx() -> usize; }

pub(crate) struct Col0;
impl ColumnIndex for Col0 { fn idx() -> usize { 0 } }
pub(crate) struct Col1;
impl ColumnIndex for Col1 { fn idx() -> usize { 1 } }
pub(crate) struct Col2;
impl ColumnIndex for Col2 { fn idx() -> usize { 2 } }
pub(crate) struct Col3;
impl ColumnIndex for Col3 { fn idx() -> usize { 3 } }
pub(crate) struct Col4;
impl ColumnIndex for Col4 { fn idx() -> usize { 4 } }
pub(crate) struct Col5;
impl ColumnIndex for Col5 { fn idx() -> usize { 5 } }

pub(crate) trait ColumnTuple: Copy {
    fn row_size(&self) -> u8;
    fn init(&self, cols: &mut [Column]);
}

pub(crate) trait ColumnTupleAccess<Col: ColumnIndex>: ColumnTuple {
    type Out: ColumnSize;
}

impl<C0: ColumnSize> ColumnTuple for (C0,) {
    fn row_size(&self) -> u8 { self.0.size() }
    fn init(&self, cols: &mut [Column]) { cols[0] = Column { offset: 0, size: self.0.size() }; }
}
impl<C0: ColumnSize> ColumnTupleAccess<Col0> for (C0,) { type Out = C0; }

impl<C0: ColumnSize, C1: ColumnSize> ColumnTuple for (C0, C1) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() }
    fn init(&self, cols: &mut [Column]) { (self.0,).init(cols); cols[1] = Column { offset: cols[0].offset + cols[0].size, size: self.1.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1) { type Out = C1; }

impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize> ColumnTuple for (C0, C1, C2) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() + self.2.size() }
    fn init(&self, cols: &mut [Column]) { (self.0, self.1).init(cols); cols[2] = Column { offset: cols[1].offset + cols[1].size, size: self.2.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1, C2) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1, C2) { type Out = C1; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize> ColumnTupleAccess<Col2> for (C0, C1, C2) { type Out = C2; }

impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTuple for (C0, C1, C2, C3) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() + self.2.size() + self.3.size() }
    fn init(&self, cols: &mut [Column]) { (self.0, self.1, self.2).init(cols); cols[3] = Column { offset: cols[2].offset + cols[2].size, size: self.3.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1, C2, C3) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1, C2, C3) { type Out = C1; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTupleAccess<Col2> for (C0, C1, C2, C3) { type Out = C2; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize> ColumnTupleAccess<Col3> for (C0, C1, C2, C3) { type Out = C3; }

impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTuple for (C0, C1, C2, C3, C4) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() + self.2.size() + self.3.size() + self.4.size() }
    fn init(&self, cols: &mut [Column]) { (self.0, self.1, self.2, self.3).init(cols); cols[4] = Column { offset: cols[3].offset + cols[3].size, size: self.4.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1, C2, C3, C4) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1, C2, C3, C4) { type Out = C1; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col2> for (C0, C1, C2, C3, C4) { type Out = C2; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col3> for (C0, C1, C2, C3, C4) { type Out = C3; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize> ColumnTupleAccess<Col4> for (C0, C1, C2, C3, C4) { type Out = C4; }

impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTuple for (C0, C1, C2, C3, C4, C5) {
    fn row_size(&self) -> u8 { self.0.size() + self.1.size() + self.2.size() + self.3.size() + self.4.size() + self.5.size() }
    fn init(&self, cols: &mut [Column]) { (self.0, self.1, self.2, self.3, self.4).init(cols); cols[5] = Column { offset: cols[4].offset + cols[4].size, size: self.5.size() }; }
}
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col0> for (C0, C1, C2, C3, C4, C5) { type Out = C0; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col1> for (C0, C1, C2, C3, C4, C5) { type Out = C1; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col2> for (C0, C1, C2, C3, C4, C5) { type Out = C2; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col3> for (C0, C1, C2, C3, C4, C5) { type Out = C3; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col4> for (C0, C1, C2, C3, C4, C5) { type Out = C4; }
impl<C0: ColumnSize, C1: ColumnSize, C2: ColumnSize, C3: ColumnSize, C4: ColumnSize, C5: ColumnSize> ColumnTupleAccess<Col5> for (C0, C1, C2, C3, C4, C5) { type Out = C5; }

pub(crate) trait ColumnSize: Copy {
    fn size(&self) -> u8;
}

#[derive(Copy, Clone, Default)]
pub(crate) struct FixedSize2;
impl ColumnSize for FixedSize2 {
    fn size(&self) -> u8 { 2 }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct FixedSize4;
impl ColumnSize for FixedSize4 {
    fn size(&self) -> u8 { 4 }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct FixedSize8;
impl ColumnSize for FixedSize8 {
    fn size(&self) -> u8 { 8 }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum DynamicSize {
    Unset,
    Size2,
    Size4
}

impl ColumnSize for DynamicSize {
    fn size(&self) -> u8 {
        match *self {
            DynamicSize::Unset => panic!("uninitialized dynamic column"),
            DynamicSize::Size2 => 2,
            DynamicSize::Size4 => 4
        }
    }
}

impl Default for DynamicSize {
    fn default() -> Self {
        DynamicSize::Unset
    }
}

#[derive(Default, Copy, Clone)]
pub(crate) struct Column {
    pub offset: u8,
    pub size: u8,
}

pub(crate) trait ColumnAccess<Col> {
    type ColumnSize: ColumnSize;
}

impl<T: TableDesc, Col: ColumnIndex> ColumnAccess<Col> for T
    where <T as TableDesc>::Columns: ColumnTupleAccess<Col>
{
    type ColumnSize = <<T as TableDesc>::Columns as ColumnTupleAccess<Col>>::Out;
}

pub(crate) trait ReadValue<S: ColumnSize> {
    fn read_value(input: &[u8], size: u8) -> Self;
}

impl ReadValue<FixedSize2> for u16 {
    fn read_value(input: &[u8], _: u8) -> Self {
        LittleEndian::read_u16(input)
    }
}

impl ReadValue<FixedSize4> for u32 {
    fn read_value(input: &[u8], _: u8) -> Self {
        LittleEndian::read_u32(input)
    }
}

impl ReadValue<FixedSize8> for u64 {
    fn read_value(input: &[u8], _: u8) -> Self {
        LittleEndian::read_u64(input)
    }
}

impl ReadValue<DynamicSize> for u32 {
    fn read_value(input: &[u8], size: u8) -> Self {
        if size == 4 {
            LittleEndian::read_u32(input)
        } else {
            LittleEndian::read_u16(input) as u32
        }
    }
}
