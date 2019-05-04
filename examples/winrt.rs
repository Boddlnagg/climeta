use std::fs;

use climeta::{Database, Cache};
use climeta::schema::{TypeDef, TypeCategory};

fn main() -> Result<(), Box<std::error::Error>> {
    let cache = Cache::new();

    for entry in fs::read_dir("C:\\Windows\\System32\\WinMetadata")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            cache.insert(Database::from_file(path)?);
        }
    }

    let mut db_count = 0;
    let mut typedef_count = 0;
    let mut enum_count = 0;
    let mut interface_count = 0;
    let mut struct_count = 0;
    let mut delegate_count = 0;
    let mut class_count = 0;
    let mut method_count = 0;

    for db in &cache {
        db_count += 1;
        for typedef in db.table::<TypeDef>() {
            typedef_count += 1;
            match typedef.type_category()? {
                TypeCategory::Enum => enum_count += 1,
                TypeCategory::Interface => interface_count += 1,
                TypeCategory::Struct => struct_count += 1,
                TypeCategory::Delegate => delegate_count += 1,
                TypeCategory::Class => class_count += 1,
            }
            for method in typedef.method_list()? {
                method_count += 1;
                let _sig = method.signature()?;
            }
        }
    }

    println!("Databases: {}, TypeDefs: {} ({} enums, {} interfaces, {} structs, {} delegates, {} classes), Methods: {}",
             db_count, typedef_count, enum_count, interface_count, struct_count, delegate_count, class_count, method_count);

    Ok(())
}
