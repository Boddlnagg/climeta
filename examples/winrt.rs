use std::fs;

use climeta::{Database, Cache};
use climeta::schema::TypeDef;

fn main() -> Result<(), Box<std::error::Error>> {
    let cache = Cache::new();
    let mut dbs = Vec::new();

    for entry in fs::read_dir("C:\\Windows\\System32\\WinMetadata")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let db = cache.insert(Database::from_file(path)?);
            dbs.push(db);
        }
    }

    let mut typedef_count = 0;
    let mut method_count = 0;

    for db in &dbs {
        for typedef in db.table::<TypeDef>() {
            typedef_count += 1;
            for method in typedef.method_list()? {
                method_count += 1;
                let _sig = method.signature()?;
            }
        }
    }

    println!("Databases: {}, TypeDefs: {}, Methods: {}", dbs.len(), typedef_count, method_count);

    Ok(())
}
