fn build_either(types: &[&str]) -> String {
    if types.len() == 1 {
        types[0].to_string()
    } else {
        format!("(Either {} {})", types[0], build_either(&types[1..]))
    }
}

fn main() {
    // 1. The AST (Rec E)
    let ast_variants = vec![
        "Int",                               // 0. Var(id)
        "(Pair E E)",                        // 1. App(f, x)
        "(Pair Int (Pair (Rec W . Either Int (Pair W W)) E))", // 2. Abs(id, type, body)
        "(Pair E E)",                        // 3. MkPair(x, y)
        "(Pair E (Pair Int (Pair Int E)))",  // 4. Unpack(target, id1, id2, body)
    ];

    // 2. The Instruction Set (Rec I)
    let inst_variants = vec![
        "Int",                               // 0. PushVar(id)
        "(Pair Int IL)",                     // 1. MakeClosure(id, body)
        "Unit",                              // 2. Call
        "Unit",                              // 3. Return
        "Unit",                              // 4. Inst::MkPair
        "(Pair Int Int)",                    // 5. Inst::Unpack(id1, id2)
    ];

    println!("--- AST Type (Rec E) ---");
    println!("Rec E . {}\n", build_either(&ast_variants));

    println!("--- Instruction Type (Rec I) ---");
    println!("Rec I . {}\n", build_either(&inst_variants));
}