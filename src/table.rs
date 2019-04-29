use crate::database::{TableDesc, ColumnIndex, ColumnTuple, ColumnTupleAccess, Database,
                      ReadValue, DynamicSize, ColumnSize};
use crate::database;
use crate::Result;

#[derive(Default, Copy, Clone, Debug)]
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

pub struct Table<'db, T: TableDesc> {
    p: std::marker::PhantomData<T>,
    m_row_count: u32,
    m_row_size: u8,
    m_columns: [Column; 6],
    m_data: Option<&'db [u8]>,
}

impl<'db, T: TableDesc> Table<'db, T> {
    pub(crate) fn set_columns<Tuple>(self: &mut Self, tup: Tuple) where T: TableDesc<Columns=Tuple>, Tuple: ColumnTuple {
        assert!(self.m_row_size == 0);
        self.m_row_size = tup.row_size();
        tup.init(&mut self.m_columns);
        //println!("{:?}", self.m_columns);
    }

    pub(crate) fn set_row_count(self: &mut Self, count: u32) {
        self.m_row_count = count;
    }

    pub fn size(&self) -> u32 {
        self.m_row_count
    }

    pub fn iter(&'db self) -> TableRowIterator<'db, T> {
        self.into_iter()
    }

    pub(crate) fn index_size(&self) -> DynamicSize {
        if self.m_row_count < (1 << 16) { DynamicSize::Size2 } else { DynamicSize::Size4 }
    }

    pub(crate) fn set_data(self: &mut Self, view: &'db [u8]) -> &'db [u8] {
        assert!(self.m_data.is_none());

        if self.m_row_count > 0 {
            assert!(self.m_row_size != 0);
            let (left, right) = view.split_at(self.m_row_count as usize * self.m_row_size as usize);
            self.m_data = Some(left);
            right
        } else {
            view
        }
    }

    pub(crate) fn get_value<Col: ColumnIndex, V>(&self, row: u32) -> Result<V>
        where T: ColumnAccess<Col>, V: ReadValue<<T as ColumnAccess<Col>>::ColumnSize>
    {
        let data_size = self.m_columns[Col::idx()].size;

        if row > self.size() {
            return Err("Invalid row index".into());
        }
        let input = &self.m_data.unwrap()[row as usize * self.m_row_size as usize + self.m_columns[Col::idx()].offset as usize ..];
        Ok(V::read_value(input, data_size))
    }

    pub fn get_row<'x>(&'x self, row: u32) -> Result<TableRow<'x, T>> {
        if row > self.size() {
            return Err("Invalid row index".into());
        }

        Ok(TableRow {
            m_table: self,
            m_row: row
        })
    }
}

impl<'db, T: TableDesc> Default for Table<'db, T> where <T as TableDesc>::Columns: Default {
   fn default() -> Self {
        Table {
            p: ::std::marker::PhantomData,
            m_row_count: 0,
            m_row_size: 0,
            m_columns: [Default::default(); 6],
            m_data: None,
        }
    }
}

impl<'db, T: TableDesc> IntoIterator for &'db Table<'db, T> {
    type Item = TableRow<'db, T>;
    type IntoIter = TableRowIterator<'db, T>;

    fn into_iter(self) -> Self::IntoIter {
        TableRowIterator {
            m_table: self,
            m_row: 0,
            m_end: self.size()
        }
    }
}

pub struct TableRow<'db, T: TableDesc> {
    m_table: &'db Table<'db, T>,
    m_row: u32,
}

pub struct TableRowIterator<'db, T: TableDesc> {
    m_table: &'db Table<'db, T>,
    m_row: u32, // the next row to yield
    m_end: u32, // end of this iterator's range (exclusive)
}

impl<'db, T: TableDesc> Iterator for TableRowIterator<'db, T> {
    type Item = TableRow<'db, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.m_row < self.m_end {
            self.m_row += 1;
            Some(self.m_table.get_row(self.m_row - 1).expect("index must be valid"))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.m_row < self.m_end {
            let size = self.m_end as usize - self.m_row as usize;
            (size, Some(size))
        } else {
            (0, Some(0))
        }
    }

    fn count(self) -> usize {
        self.m_end as usize - self.m_row as usize
    }

    fn last(self) -> Option<Self::Item> {
        if self.m_row < self.m_end {
            Some(self.m_table.get_row(self.m_end - 1).expect("index must be valid"))
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.m_row + (n as u32) < self.m_end {
            Some(self.m_table.get_row(self.m_row + (n as u32)).expect("index must be valid"))
        } else {
            None
        }
    }
}

impl<'db, T: TableDesc> TableRow<'db, T> {
    pub fn index(&self) -> u32 { self.m_row }

    pub(crate) fn get_value<Col: ColumnIndex, V>(&self) -> Result<V>
        where T: ColumnAccess<Col>, V: ReadValue<<T as ColumnAccess<Col>>::ColumnSize>
    {
        self.m_table.get_value::<Col, _>(self.m_row)
    }

    pub(crate) fn get_string<Col: ColumnIndex>(&self, db: &'db Database) -> Result<&'db str>
        where T: ColumnAccess<Col>, u32: ReadValue<<T as ColumnAccess<Col>>::ColumnSize>
    {
        db.get_string(self.get_value::<Col, _>()?)
    }

    pub(crate) fn get_blob<Col: ColumnIndex>(&self, db: &'db Database) -> Result<&'db [u8]>
        where T: ColumnAccess<Col>, u32: ReadValue<<T as ColumnAccess<Col>>::ColumnSize>
    {
        db.get_blob(self.get_value::<Col, _>()?)
    }

    pub(crate) fn get_coded_index<Col: ColumnIndex, Target: database::CodedIndex>(&self, db: Target::Database) -> Result<Option<Target>>
        where T: ColumnAccess<Col>, u32: ReadValue<<T as ColumnAccess<Col>>::ColumnSize>
    {
        Target::decode(self.get_value::<Col, _>()?, db)
    }

    pub(crate) fn get_list<Col: ColumnIndex, Target: TableDesc>(&self, db: &'db Database<'db>) -> Result<TableRowIterator<'db, Target>>
        where database::Tables<'db>: database::TableAccess<'db, Target>,
              T: ColumnAccess<Col>, u32: ReadValue<<T as ColumnAccess<Col>>::ColumnSize>
    {
        let target_table = db.get_table::<Target>();
        let first = self.get_value::<Col, u32>()?;
        assert!(first != 0);
        let first = first - 1;

        let last = if self.m_row + 1 < self.m_table.size() {
            // this is not the last row
            let tmp = self.m_table.get_row(self.m_row + 1)?.get_value::<Col, u32>()?;
            assert!(tmp != 0);
            tmp - 1
        } else {
            target_table.size()
        };
        
        Ok(TableRowIterator {
            m_table: target_table,
            m_row: first,
            m_end: last
        })
    }

    pub(crate) fn get_target_row<Col: ColumnIndex, Target: TableDesc>(&self, tables: &'db database::Tables<'db>)  -> Result<TableRow<'db, Target>>
        where database::Tables<'db>: database::TableAccess<'db, Target>,
              T: ColumnAccess<Col>, u32: ReadValue<<T as ColumnAccess<Col>>::ColumnSize>
    {
        let target_table = tables.get_table::<Target>();
        let row = self.get_value::<Col, u32>()?;
        assert!(row != 0);
        target_table.get_row(row - 1)
    }
}