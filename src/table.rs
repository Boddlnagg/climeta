use crate::database::{TableDesc, TableKind, ColumnIndex, ColumnTupleAccess, Database,
                      ReadValue, ColumnSize};
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

#[derive(Copy, Clone)]
pub struct Table<'db, T: TableKind> {
    pub(crate) db: &'db database::Database<'db>,
    pub(crate) table: &'db database::TableInfo<'db, T>,
}

impl<'db, T: TableKind> Table<'db, T> {
    pub fn size(&self) -> u32 {
        self.table.m_row_count
    }

    pub fn iter(&self) -> TableRowIterator<'db, T> {
        self.into_iter()
    }

    pub(crate) fn get_value<Col: ColumnIndex, V>(&self, row: u32) -> Result<V>
        where T: ColumnAccess<Col>, V: ReadValue<T::ColumnSize>
    {
        let data_size = self.table.m_columns[Col::idx()].size;

        if row > self.size() {
            return Err("Invalid row index".into());
        }
        let input = &self.table.m_data.unwrap()[row as usize * self.table.m_row_size as usize +
                                                self.table.m_columns[Col::idx()].offset as usize ..];
        Ok(V::read_value(input, data_size))
    }

    pub fn get_row(&self, row: u32) -> Result<TableRow<'db, T>> {
        if row > self.size() {
            return Err("Invalid row index".into());
        }

        Ok(TableRow {
            m_table: *self,
            m_row: row
        })
    }
}

impl<'db, T: TableKind> IntoIterator for Table<'db, T> {
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

pub struct TableRow<'db, T: TableKind> {
    m_table: Table<'db, T>,
    m_row: u32,
}

pub struct TableRowIterator<'db, T: TableKind> {
    m_table: Table<'db, T>,
    m_row: u32, // the next row to yield
    m_end: u32, // end of this iterator's range (exclusive)
}

impl<'db, T: TableKind> Iterator for TableRowIterator<'db, T> {
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

impl<'db, T: TableKind> TableRow<'db, T> {
    pub fn index(&self) -> u32 { self.m_row }

    pub(crate) fn get_value<Col: ColumnIndex, V>(&self) -> Result<V>
        where T: ColumnAccess<Col>, V: ReadValue<T::ColumnSize>
    {
        self.m_table.get_value::<Col, _>(self.m_row)
    }

    pub(crate) fn get_string<Col: ColumnIndex>(&self) -> Result<&'db str>
        where T: ColumnAccess<Col>, u32: ReadValue<T::ColumnSize>
    {
        self.m_table.db.get_string(self.get_value::<Col, _>()?)
    }

    pub(crate) fn get_blob<Col: ColumnIndex>(&self) -> Result<&'db [u8]>
        where T: ColumnAccess<Col>, u32: ReadValue<T::ColumnSize>
    {
        self.m_table.db.get_blob(self.get_value::<Col, _>()?)
    }

    pub(crate) fn get_coded_index<Col: ColumnIndex, Target: database::CodedIndex<Database=&'db Database<'db>>>(&self) -> Result<Option<Target>>
        where T: ColumnAccess<Col>, u32: ReadValue<T::ColumnSize>
    {
        Target::decode(self.get_value::<Col, _>()?, self.m_table.db)
    }

    pub(crate) fn get_list<Col: ColumnIndex, Target: TableKind>(&self) -> Result<TableRowIterator<'db, Target>>
        where database::Database<'db>: database::TableAccess<'db, Target>,
              T: ColumnAccess<Col>, u32: ReadValue<T::ColumnSize>
    {
        let target_table = self.m_table.db.get_table::<Target>();
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

    pub(crate) fn get_target_row<Col: ColumnIndex, Target: TableKind>(&self)  -> Result<TableRow<'db, Target>>
        where database::Database<'db>: database::TableAccess<'db, Target>,
              T: ColumnAccess<Col>, u32: ReadValue<T::ColumnSize>
    {
        let target_table = self.m_table.db.get_table::<Target>();
        let row = self.get_value::<Col, u32>()?;
        assert!(row != 0);
        target_table.get_row(row - 1)
    }
}