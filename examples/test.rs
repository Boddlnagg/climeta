use climeta::{database, schema};

fn main() -> Result<(), Box<std::error::Error>> {
    println!("=== Windows.Foundation.winmd ===");
    let f1 = database::mmap_file("C:\\Windows\\System32\\WinMetadata\\Windows.Foundation.winmd").unwrap();
    let (db, tab) = database::Database::load(&f1).unwrap();
    let typedefs = tab.get_table::<schema::TypeDef>();
    for row in typedefs {
        println!("{}.{}", row.type_namespace(&db)?, row.type_name(&db)?);

        
        for md in row.method_list(&tab)? {
            println!(" M {}", md.name(&db)?);
            for mpar in md.param_list(&tab)? {
                println!("   P {} {}", mpar.sequence()?, mpar.name(&db)?);
            }
        }
        // for fld in row.field_list(&tab)? {
        //     println!(" F {}", fld.name(&db)?);
        // }

        match row.extends(&tab)? {
            None => println!(" Extends: <None>"),
            Some(schema::TypeDefOrRef::TypeDef(def)) => {
                println!(" Extends: {}.{} (def)", def.type_namespace(&db)?, def.type_name(&db)?);
            },
            Some(schema::TypeDefOrRef::TypeRef(def)) => {
                println!(" Extends: {}.{} (ref: {:?}) ", def.type_namespace(&db)?, def.type_name(&db)?, def.resolution_scope(&tab)?);
            },
            _ => ()
        }
    }
    println!("TOTAL: {} == {}", typedefs.size(), typedefs.into_iter().count());

    // for cons in tab.get_table::<schema::Constant>() {
    //     let parent = cons.parent(&tab)?;
    //     println!("{:?}, parent: {:?}", cons.typ()?, parent);
    //     if let Some(schema::HasConstant::Field(f)) = parent {
    //         println!("  {} -> {:?}", f.name(&db)?, cons.value(&db)?);
    //     }
    // }
    
    println!("=== Windows.UI.Xaml.winmd ===");
    let f2 = database::mmap_file("C:\\Windows\\System32\\WinMetadata\\Windows.UI.Xaml.winmd").unwrap();
    let (db, tab) = database::Database::load(&f2).unwrap();
    for cons in tab.get_table::<schema::Constant>() {
        
    }
    //let typedefs = tab.get_table::<schema::TypeDef>();
    // for row in typedefs {
    //     println!("{}.{}", row.type_namespace(&db)?, row.type_name(&db)?);

        
    //     // for md in row.method_list(&tab)? {
    //     //     println!(" M {}", md.name(&db)?);
    //     // }
    //     // for fld in row.field_list(&tab)? {
    //     //     println!(" F {}", fld.name(&db)?);
    //     // }

    //     match row.extends(&tab)? {
    //         None => println!("  Extends: <None>"),
    //         Some(schema::TypeDefOrRef::TypeDef(def)) => println!("  Extends: {}.{} (def)", def.type_namespace(&db)?, def.type_name(&db)?),
    //         Some(schema::TypeDefOrRef::TypeRef(def)) => println!("  Extends: {}.{} (ref)", def.type_namespace(&db)?, def.type_name(&db)?),
    //         _ => ()
    //     }
    // }
    // println!("TOTAL: {}", typedefs.size());

    Ok(())
}