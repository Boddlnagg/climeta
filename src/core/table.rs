use crate::database::{TableDesc, TableKind, Database};
use crate::database;
use crate::schema::TableRowAccess;
use crate::Result;

use crate::core::columns::{Column, ColumnIndex, ColumnTuple, ColumnAccess, ReadValue, DynamicSize};

// TODO: not pub?
pub struct TableInfo<'db, T> {
    p: std::marker::PhantomData<T>,
    pub(crate) m_row_count: u32,
    pub(crate) m_row_size: u8,
    pub(crate) m_columns: [Column; 6],
    pub(crate) m_data: Option<&'db [u8]>,
}

impl<'db, T> TableInfo<'db, T> {
    pub(crate) fn set_columns<Tuple>(&mut self, tup: Tuple) where T: TableDesc<Columns=Tuple>, Tuple: ColumnTuple
    {
        assert!(self.m_row_size == 0);
        self.m_row_size = tup.row_size();
        tup.init(&mut self.m_columns);
        //println!("{:?}", self.m_columns);
    }

    pub(crate) fn set_row_count(&mut self, count: u32) {
        self.m_row_count = count;
    }

    pub(crate) fn index_size(&self) -> DynamicSize {
        if self.m_row_count < (1 << 16) { DynamicSize::Size2 } else { DynamicSize::Size4 }
    }

    pub(crate) fn set_data(&mut self, view: &'db [u8]) -> &'db [u8]
    {
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
}

impl<'db, T> Default for TableInfo<'db, T> {
   fn default() -> Self {
        TableInfo {
            p: ::std::marker::PhantomData,
            m_row_count: 0,
            m_row_size: 0,
            m_columns: [Default::default(); 6],
            m_data: None,
        }
    }
}

#[derive(Copy, Clone)]
// TODO: try to use TableRow parameter instead of TableKind
pub struct Table<'db, T: TableKind> {
    pub(crate) db: &'db database::Database<'db>,
    pub(crate) table: &'db TableInfo<'db, T>,
}

impl<'db, T: TableKind> Table<'db, T> where &'db T: TableRowAccess<Table=Self> {
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

    pub fn get_row(&self, row: u32) -> Result<<&'db T as TableRowAccess>::Out>
    {
        if row > self.size() {
            return Err("Invalid row index".into());
        }
        
        Ok(<&'db T as TableRowAccess>::get(*self, row))
    }
}

impl<'db, T: TableKind> IntoIterator for Table<'db, T>
    where &'db T: TableRowAccess<Table=Table<'db, T>>
{
    type Item = <&'db T as TableRowAccess>::Out;
    type IntoIter = TableRowIterator<'db, T>;

    fn into_iter(self) -> Self::IntoIter {
        TableRowIterator {
            m_table: self,
            m_row: 0,
            m_end: self.size()
        }
    }
}

pub(crate) struct Row<'db, T: TableKind> {
    m_table: Table<'db, T>,
    m_row: u32,
}

pub struct TableRowIterator<'db, T: TableKind> {
    m_table: Table<'db, T>,
    m_row: u32, // the next row to yield
    m_end: u32, // end of this iterator's range (exclusive)
}

impl<'db, T: TableKind> Iterator for TableRowIterator<'db, T>
    where &'db T: TableRowAccess<Table=Table<'db, T>>
{
    type Item = <&'db T as TableRowAccess>::Out;

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

impl<'db, T: TableKind> Row<'db, T> where &'db T: TableRowAccess<Table=Table<'db, T>> {
    pub(crate) fn new(table: Table<'db, T>, row: u32) -> Row<'db, T> {
        Row {
            m_table: table,
            m_row: row
        }
    }
    pub(crate) fn index(&self) -> u32 { self.m_row }

    pub(crate) fn get_db(&self) -> &'db Database {
        self.m_table.db
    }

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
              T: ColumnAccess<Col>, u32: ReadValue<T::ColumnSize>,
              &'db Target: TableRowAccess<Table=Table<'db, Target>>,
              <&'db Target as TableRowAccess>::Out: crate::schema::TableRow<Kind=Target>,
    {
        let target_table = self.m_table.db.get_table::<<&'db Target as TableRowAccess>::Out>();
        let first = self.get_value::<Col, u32>()?;
        assert!(first != 0);
        let first = first - 1;

        let last = if self.m_row + 1 < self.m_table.size() {
            // this is not the last row
            let tmp = self.m_table.get_value::<Col, u32>(self.m_row + 1)?;
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

    pub(crate) fn get_target_row<Col: ColumnIndex, Target: TableKind>(&self)  -> Result<<&'db Target as TableRowAccess>::Out>
        where database::Database<'db>: database::TableAccess<'db, Target>,
              T: ColumnAccess<Col>, u32: ReadValue<T::ColumnSize>,
              &'db Target: TableRowAccess<Table=Table<'db, Target>>,
              <&'db Target as TableRowAccess>::Out: crate::schema::TableRow<Kind=Target>
    {
        let target_table = self.m_table.db.get_table::<<&'db Target as TableRowAccess>::Out>();
        let row = self.get_value::<Col, u32>()?;
        assert!(row != 0);
        target_table.get_row(row - 1)
    }
}