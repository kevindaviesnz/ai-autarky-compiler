use std::fs;

fn build_either(types: &[String]) -> String {
    if types.len() == 1 {
        types[0].clone()
    } else {
        format!("(Either {} {})", types[0], build_either(&types[1..]))
    }
}

fn make_inst(variant_idx: usize, payload: &str, t_i: &str, t_il: &str) -> String {
    let unrolled = vec![
        "Int".to_string(),
        format!("(Pair Int {})", t_il),
        "Unit".to_string(),
        "Unit".to_string(),
        "Unit".to_string(),
        "(Pair Int Int)".to_string(),
        "Unit".to_string(),
        "Unit".to_string(),
        format!("(Pair {} {})", t_il, t_il),
    ];
    let mut current = payload.to_string();
    if variant_idx == unrolled.len() - 1 {
    } else {
        let rest = &unrolled[variant_idx + 1..];
        let either_rest = build_either(rest);
        current = format!("(Left {} {})", current, either_rest);
    }
    for i in (0..variant_idx).rev() {
        current = format!("(Right {} {})", unrolled[i], current);
    }
    format!("(fold [{}] {})", t_i, current)
}

fn make_expr(variant_idx: usize, payload: &str, t_e: &str, t_w: &str) -> String {
    let types = vec![
        "Int".to_string(),
        "(Pair __E__ __E__)".to_string(),
        format!("(Pair Int (Pair {} __E__))", t_w),
        "(Pair __E__ __E__)".to_string(),
        "(Pair __E__ (Pair Int (Pair Int __E__)))".to_string(),
        "__E__".to_string(),
        "__E__".to_string(),
        "(Pair __E__ (Pair Int (Pair __E__ (Pair Int __E__))))".to_string(),
    ];
    let unrolled: Vec<String> = types.iter().map(|s| s.replace("__E__", t_e)).collect();
    let mut current = payload.to_string();
    if variant_idx == unrolled.len() - 1 {
    } else {
        let rest_types = &unrolled[variant_idx + 1..];
        let either_rest = build_either(rest_types);
        current = format!("(Left {} {})", current, either_rest);
    }
    for i in (0..variant_idx).rev() {
        current = format!("(Right {} {})", unrolled[i], current);
    }
    format!("(fold [{}] {})", t_e, current)
}

fn main() {
    let t_w = "(Rec W . Either Int (Pair W W))".to_string();
    
    let ast_variants = vec![
        "Int".to_string(), "(Pair __E__ __E__)".to_string(), format!("(Pair Int (Pair {} __E__))", t_w),
        "(Pair __E__ __E__)".to_string(), "(Pair __E__ (Pair Int (Pair Int __E__)))".to_string(),
        "__E__".to_string(), "__E__".to_string(), "(Pair __E__ (Pair Int (Pair __E__ (Pair Int __E__))))".to_string(),
    ];
    let t_e_body = build_either(&ast_variants).replace("__E__", "E");
    let t_e = format!("(Rec E . {})", t_e_body);

    let t_il_rec = "(Rec IL . Either Unit (Pair I IL))".to_string();
    let inst_variants = vec![
        "Int".to_string(), format!("(Pair Int {})", t_il_rec), "Unit".to_string(), "Unit".to_string(),
        "Unit".to_string(), "(Pair Int Int)".to_string(), "Unit".to_string(), "Unit".to_string(),
        format!("(Pair {} {})", t_il_rec, t_il_rec),
    ];
    let t_i_body = build_either(&inst_variants);
    let t_i = format!("(Rec I . {})", t_i_body);
    let t_il = format!("(Rec IL . Either Unit (Pair {} IL))", t_i);
    let t_l = format!("(Rec L . Either Unit (Pair (Pair Int {}) L))", t_w);

    let v_120 = make_expr(0, "120", &t_e, &t_w);
    let in_left = make_expr(5, &v_120, &t_e, &t_w);
    let v_x = make_expr(0, "1", &t_e, &t_w);
    let v_y = make_expr(0, "2", &t_e, &t_w);
    let match_payload = format!("(mkpair {} (mkpair 1 (mkpair {} (mkpair 2 {}))))", in_left, v_x, v_y);
    let mock_expr = make_expr(7, &match_payload, &t_e, &t_w);

    let inst_pushvar = make_inst(0, "var_id", &t_i, &t_il);
    let inst_makeleft = make_inst(6, "unit", &t_i, &t_il);
    let inst_makeright = make_inst(7, "unit", &t_i, &t_il);
    let inst_branchmatch = make_inst(8, "(mkpair l_code r_code)", &t_i, &t_il);

    let concat_def = format!("( fix ( \\concat : Pi l1 : {t_il} . Pi l2 : {t_il} . {t_il} . \\l1 : {t_il} . \\l2 : {t_il} . match (unfold l1) with Left empty => l2 | Right cnode => unpack cnode into head, tail in (fold [{t_il}] (Right Unit (mkpair head ( ( (concat tail) l2 ) )))) ) )");

    // Pre-load the mock context with [120 : Int] so the VM doesn't panic
    let t_int = format!("(fold [{t_w}] (Left 1 (Pair {t_w} {t_w})))");
    let empty_ctx = format!("(fold [{t_l}] (Left unit (Pair (Pair Int {t_w}) {t_l})))");
    let mock_ctx = format!("(fold [{t_l}] (Right Unit (mkpair (mkpair 120 {t_int}) {empty_ctx})))");

    let template = format!(r#"
(
  ( \compile : Pi ctx : {t_l} . Pi expr : {t_e} . Pair (Pair (Either Unit {t_w}) {t_l}) {t_il} .
    (
      ( \mock_ctx : {t_l} .
        (
          ( \mock_expr : {t_e} .
            ((compile mock_ctx) mock_expr)
          )
          {mock_expr}
        )
      )
      {mock_ctx}
    )
  )
  ( fix
    ( \comp : Pi ctx : {t_l} . Pi expr : {t_e} . Pair (Pair (Either Unit {t_w}) {t_l}) {t_il} .
      \ctx : {t_l} . \expr : {t_e} .
        match (unfold expr) with
          Left var_id => 
            unpack ( ( ( ( fix ( \lkp : Pi c : {t_l} . Pi t : Int . Pair (Either Unit {t_w}) {t_l} . \c : {t_l} . \t : Int . match (unfold c) with Left empty => mkpair (Left unit {t_w}) (fold [{t_l}] (Left unit (Pair (Pair Int {t_w}) {t_l}))) | Right node => unpack node into binding, rest in unpack binding into name, type_val in if (name == t) then mkpair (Right Unit type_val) rest else unpack ((lkp rest) t) into result_type, next_ctx in mkpair result_type (fold [{t_l}] (Right Unit (mkpair (mkpair name type_val) next_ctx))) ) ) ctx ) var_id ) ) into res_type, next_ctx in
            mkpair (mkpair res_type next_ctx) (fold [{t_il}] (Right Unit (mkpair {inst_pushvar} (fold [{t_il}] (Left unit (Pair {t_i} {t_il}))))))
        | Right node1 => match node1 with
            Left app_node => mkpair (mkpair (Left unit {t_w}) ctx) (fold [{t_il}] (Left unit (Pair {t_i} {t_il})))
          | Right node2 => match node2 with
              Left abs_node => mkpair (mkpair (Left unit {t_w}) ctx) (fold [{t_il}] (Left unit (Pair {t_i} {t_il})))
            | Right node3 => match node3 with
                Left mkpair_node => mkpair (mkpair (Left unit {t_w}) ctx) (fold [{t_il}] (Left unit (Pair {t_i} {t_il})))
              | Right node4 => match node4 with
                  Left unpack_node => mkpair (mkpair (Left unit {t_w}) ctx) (fold [{t_il}] (Left unit (Pair {t_i} {t_il})))
                | Right node5 => match node5 with
                    Left inleft_node => 
                      unpack ((comp ctx) inleft_node) into res, code in
                        mkpair res ( ( {concat_def} code ) (fold [{t_il}] (Right Unit (mkpair {inst_makeleft} (fold [{t_il}] (Left unit (Pair {t_i} {t_il})))))) )
                  | Right node6 => match node6 with
                      Left inright_node => 
                        unpack ((comp ctx) inright_node) into res, code in
                          mkpair res ( ( {concat_def} code ) (fold [{t_il}] (Right Unit (mkpair {inst_makeright} (fold [{t_il}] (Left unit (Pair {t_i} {t_il})))))) )
                    | Right match_node => 
                        unpack match_node into tgt_expr, r1 in
                        unpack r1 into id_l, r2 in
                        unpack r2 into b_l, r3 in
                        unpack r3 into id_r, b_r in
                          unpack ((comp ctx) tgt_expr) into tgt_tc, tgt_code in
                          unpack tgt_tc into tgt_res, ctx_after_tgt in
                            unpack ((comp (fold [{t_l}] (Right Unit (mkpair (mkpair id_l (fold [{t_w}] (Left 1 (Pair {t_w} {t_w})))) ctx_after_tgt)))) b_l) into l_tc, l_code in
                            unpack l_tc into l_res, ctx_after_l in
                            unpack ((comp (fold [{t_l}] (Right Unit (mkpair (mkpair id_r (fold [{t_w}] (Left 1 (Pair {t_w} {t_w})))) ctx_after_tgt)))) b_r) into r_tc, r_code in
                            unpack r_tc into r_res, ctx_after_r in
                              mkpair (mkpair l_res ctx_after_l)
                                ( ( {concat_def} tgt_code ) (fold [{t_il}] (Right Unit (mkpair {inst_branchmatch} (fold [{t_il}] (Left unit (Pair {t_i} {t_il})))))) )
    )
  )
)
"#);

    fs::write("main.aut", template).unwrap();
    println!("Stage 8 Bootstrapper: Postfix Branching Engine Generated!");
}