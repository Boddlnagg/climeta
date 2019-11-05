use climeta::schema::{Module, RetTypeKind, Type, TypeDef, TypeDefOrRef};
use climeta::{Cache, Database, ResolveToTypeDef};

pub fn mmap_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<memmap::Mmap> {
    let file = std::fs::File::open(path.as_ref())?;
    unsafe { memmap::Mmap::map(&file) }
}

fn print_typedef<'x, 'c, 'db: 'c>(
    row: &'x TypeDef<'db>, cache: &'c Cache<'db>
) -> Result<(), Box<std::error::Error>> {
    println!(
        "{}.{} ({:?})",
        row.type_namespace()?,
        row.type_name()?,
        row.flags()?.semantics()
    );

    for md in row.method_list()? {
        let sig = md.signature()?;

        println!(" - M {} with {} param(s)", md.name()?, sig.params().len());

        for (mpar, mpar_t) in md
            .param_list()?
            .skip_while(|p| {
                if let Ok(0) = p.sequence() {
                    true
                } else {
                    false
                }
            })
            .zip(sig.params())
        {
            let flags = mpar.flags()?;
            let inout = match (flags.in_(), flags.out()) {
                (true, true) => "In/Out", // probably never happens
                (true, false) => "In",
                (false, true) => "Out",
                (false, false) => "-" // happens for constructors
            };
            println!(
                "   - P {} {}: [{}] {:?}",
                mpar.sequence()?,
                mpar.name()?,
                inout,
                mpar_t.kind()
            );
        }

        let ret = sig.return_type().kind();
        match ret {
            RetTypeKind::Type(Type::Ref(_, TypeDefOrRef::TypeRef(t), _)) => {
                println!("   - R {}.{}", t.type_namespace()?, t.type_name()?)
            },
            _ => println!("   - R {:?}", ret)
        }
    }
    // for fld in row.field_list()? {
    //     println!(" F {}", fld.name()?);
    // }

    match row.extends()? {
        None => println!(" Extends: <None>"),
        Some(TypeDefOrRef::TypeDef(def)) => {
            println!(
                " Extends: {}.{} (def)",
                def.type_namespace()?,
                def.type_name()?
            );
        },
        Some(TypeDefOrRef::TypeRef(def)) => {
            println!(
                " Extends: {}.{} (ref: {:?}) ",
                def.type_namespace()?,
                def.type_name()?,
                def.resolution_scope()?
            );
        },
        _ => ()
    }

    for intf in row.interface_impls()? {
        match intf.interface()? {
            TypeDefOrRef::TypeDef(def) => {
                println!(
                    " Implements: {}.{} (def)",
                    def.type_namespace()?,
                    def.type_name()?
                );
            },
            TypeDefOrRef::TypeRef(def) => {
                println!(
                    " Implements: {}.{} (ref: {:?}) ",
                    def.type_namespace()?,
                    def.type_name()?,
                    def.resolution_scope()?
                );
            },
            TypeDefOrRef::TypeSpec(spec) => {
                println!(" Implements: {:?} (spec)", spec.signature()?);
            }
        }
    }

    println!("!!! Attributes of {:?}:", row);
    for attr in row.custom_attributes()? {
        let ty = attr.type_()?;
        match ty {
            climeta::schema::CustomAttributeType::MethodDef(md) => println!(" - MethodDef"),
            climeta::schema::CustomAttributeType::MemberRef(mr) => {
                let parent = mr.class()?;
                match parent {
                    climeta::schema::MemberRefParent::TypeRef(tr) => println!(" - {:?}", tr),
                    _ => println!(" - ???")
                }
                //println!("MemberRef {}", mr.name()?)
            }
        }
        let val = attr.value(cache)?;
        //println!("   -> {:?}", val.fixed_args());
        if val.named_args().len() > 0 {
            println!("   -> {:?}", val.named_args());
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<std::error::Error>> {
    println!("=== Windows.Foundation.winmd ===");
    let cache = Cache::new();
    let db = cache.insert(Database::from_file(
        "C:\\Windows\\System32\\WinMetadata\\Windows.Foundation.winmd"
    )?);
    let typedefs = db.table::<TypeDef>();
    for row in typedefs {
        print_typedef(&row, &cache)?;
    }
    let modules = db.table::<Module>();
    println!("TOTAL: {} == {}", typedefs.len(), typedefs.iter().count());
    println!("Looking for Windows.Foundation.Point ...");
    print_typedef(&cache.find("Windows.Foundation", "Point").unwrap(), &cache)?;
    print_typedef(&"Windows.Foundation.Point".resolve(&cache).unwrap(), &cache)?;

    // println!("Typespecs:");
    // for ts in db.get_table::<TypeSpec>() {
    //     println!("- {:?}", ts.signature()?);
    // }

    // for cons in db.get_table::<schema::Constant>() {
    //     let parent = cons.parent()?;
    //     println!("{:?}, parent: {:?}", cons.type_()?, parent);
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
    let db2 = cache.insert(Database::from_file(
        "C:\\Windows\\System32\\WinMetadata\\Windows.UI.Xaml.winmd"
    )?);
    //let f2 = mmap_file("C:\\Windows\\System32\\WinMetadata\\Windows.UI.Xaml.winmd").unwrap();
    let typedefs = db2.table::<TypeDef>();
    for row in typedefs {
        //print_typedef(&row)?;
    }

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
    //         Some(TypeDefOrRef::TypeDef(def)) => println!("  Extends: {}.{} (def)", def.type_namespace()?, def.type_name()?),
    //         Some(TypeDefOrRef::TypeRef(def)) => println!("  Extends: {}.{} (ref)", def.type_namespace()?, def.type_name()?),
    //         _ => ()
    //     }
    // }

    println!("TOTAL: {}", typedefs.len());
    println!("TOTAL (Foundation): {}", db.table::<TypeDef>().len());

    assert_eq!(1, modules.len());

    println!("Module (Foundation): {}", modules.get_row(0)?.name()?);

    Ok(())
}
