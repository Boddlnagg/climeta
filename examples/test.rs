use climeta::{database, schema};

fn print_typedef(row: &database::TableRow<schema::TypeDef>) -> Result<(), Box<std::error::Error>> {
    println!("{}.{} ({:?})", row.type_namespace()?, row.type_name()?, row.flags()?.semantics());

    for md in row.method_list()? {
        println!(" M {}", md.name()?);
        for mpar in md.param_list()? {
            let flags = mpar.flags()?;
            let inout = match (flags.in_(), flags.out()) {
                (true, true) => "in/out",
                (true, false) => "in",
                (false, true) => "out",
                (false, false) => "-"
            };
            println!("   P {} {} ({})", mpar.sequence()?, mpar.name()?, inout);
        }
    }
    // for fld in row.field_list()? {
    //     println!(" F {}", fld.name()?);
    // }

    match row.extends()? {
        None => println!(" Extends: <None>"),
        Some(schema::TypeDefOrRef::TypeDef(def)) => {
            println!(" Extends: {}.{} (def)", def.type_namespace()?, def.type_name()?);
        },
        Some(schema::TypeDefOrRef::TypeRef(def)) => {
            println!(" Extends: {}.{} (ref: {:?}) ", def.type_namespace()?, def.type_name()?, def.resolution_scope()?);
        },
        _ => ()
    }

    Ok(())
}

fn main() -> Result<(), Box<std::error::Error>> {
    println!("=== Windows.Foundation.winmd ===");
    let f1 = database::mmap_file("C:\\Windows\\System32\\WinMetadata\\Windows.Foundation.winmd").unwrap();
    let db = database::Database::load(&f1).unwrap();
    let typedefs = db.get_table::<schema::TypeDef>();
    for row in typedefs {
        print_typedef(&row)?;
    }
    for i in 0..typedefs.size() {
        println!("== {} ==", i);
        print_typedef(&typedefs.get_row(i)?)?;
        print_typedef(&typedefs.iter().nth(i as usize).unwrap())?;
    }
    println!("TOTAL: {} == {}", typedefs.size(), typedefs.iter().count());

    // for cons in db.get_table::<schema::Constant>() {
    //     let parent = cons.parent()?;
    //     println!("{:?}, parent: {:?}", cons.typ()?, parent);
    //     if let Some(schema::HasConstant::Field(f)) = parent {
    //         println!("  {} -> {:?}", f.name()?, cons.value()?);
    //     }
    // }

    // for ms in db.get_table::<schema::MethodSemantics>() {
    //     let meth = ms.method()?;
    //     let sem = if ms.semantics()?.getter() {
    //         "getter"
    //     } else if ms.semantics()?.setter() {
    //         "setter"
    //     } else {
    //         "..."
    //     };
    //     println!("Semantics for method {:?}: {:?}", meth.name()?, sem);
    // }
    
    println!("=== Windows.UI.Xaml.winmd ===");
    let f2 = database::mmap_file("C:\\Windows\\System32\\WinMetadata\\Windows.UI.Xaml.winmd").unwrap();
    let db = database::Database::load(&f2).unwrap();
    for cons in db.get_table::<schema::Constant>() {
        
    }
    //let typedefs = db.get_table::<schema::TypeDef>();
    // for row in typedefs {
    //     println!("{}.{}", row.type_namespace()?, row.type_name()?);

        
    //     // for md in row.method_list()? {
    //     //     println!(" M {}", md.name()?);
    //     // }
    //     // for fld in row.field_list()? {
    //     //     println!(" F {}", fld.name()?);
    //     // }

    //     match row.extends()? {
    //         None => println!("  Extends: <None>"),
    //         Some(schema::TypeDefOrRef::TypeDef(def)) => println!("  Extends: {}.{} (def)", def.type_namespace()?, def.type_name()?),
    //         Some(schema::TypeDefOrRef::TypeRef(def)) => println!("  Extends: {}.{} (ref)", def.type_namespace()?, def.type_name()?),
    //         _ => ()
    //     }
    // }
    // println!("TOTAL: {}", typedefs.size());

    Ok(())
}