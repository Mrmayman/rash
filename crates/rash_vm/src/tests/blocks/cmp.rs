use crate::{
    Ptr,
    builder::{c_if_else, equal, gt, lt, set},
    tests::blocks::{set_vars, utils::run_code},
};

#[test]
fn comparison_const_const() {
    let memory = run_code(set_vars(vec![
        // --- Numbers ---
        gt(3.0, 2.0).into(),    // true
        gt(2.0, 3.0).into(),    // false
        gt(3.0, 3.0).into(),    // false
        lt(3.0, 2.0).into(),    // false
        lt(2.0, 3.0).into(),    // true
        lt(3.0, 3.0).into(),    // false
        equal(3.0, 3.0).into(), // true
        equal(3.0, 2.0).into(), // false
        // --- Strings (byte-wise compare, case insensitive) ---
        lt("apple", "banana").into(),
        gt("banana", "apple").into(),
        equal("apple", "apple").into(),
        lt("Apple", "apple").into(), // 'A'(65) < 'a'(97)
        // --- Numeric strings (parsed as numbers) ---
        lt("5", "10").into(), // 5 < 10
        gt("10", "5").into(),
        // --- Booleans ---
        gt(true, false).into(),
        lt(false, true).into(),
        equal(true, true).into(),
        equal(false, false).into(),
        // --- Bool vs Number (bool coerced to 1.0 / 0.0) ---
        equal(true, 1.0).into(),
        equal(false, 0.0).into(),
        gt(true, 0.5).into(),
        // --- Regression: -0.0 vs 0.0 (regular IEEE, they are equal) ---
        equal(-0.0, 0.0).into(),  // true
        lt(-0.0, 0.0).into(),     // false
        gt(-0.0, 0.0).into(),     // false
        equal(0.0, -0.0).into(),  // true
        equal(-0.0, -0.0).into(), // true
        // --- Regression: NaN == NaN ---
        equal(f64::NAN, f64::NAN).into(), // true
        gt(f64::NAN, f64::NAN).into(),    // false
        lt(f64::NAN, f64::NAN).into(),    // false
        // --- Regression: NaN > all non-NaN ---
        gt(f64::NAN, 5.0).into(),               // true
        lt(f64::NAN, 5.0).into(),               // false
        equal(f64::NAN, 5.0).into(),            // false
        lt(5.0, f64::NAN).into(),               // true
        gt(5.0, f64::NAN).into(),               // false
        equal(5.0, f64::NAN).into(),            // false
        gt(f64::NAN, f64::INFINITY).into(),     // true
        lt(f64::NEG_INFINITY, f64::NAN).into(), // true
        gt(f64::NAN, f64::NEG_INFINITY).into(), // true
    ]));

    let expected = [
        true, false, false, false, true, false, true, false, // numbers
        true, true, true, false, // strings
        true, true, // numeric strings
        true, true, true, true, // booleans
        true, true, true, // bool vs number
        true, false, false, true, true, // -0.0 vs 0.0
        true, false, false, // NaN == NaN
        true, false, false, true, false, false, true, true, true, // NaN > non-NaN
    ];
    for (i, want) in expected.into_iter().enumerate() {
        assert_eq!(memory[i].convert_to_bool(), want, "index {i}");
    }
}

#[test]
fn comparison_fast_path() {
    let memory = run_code(vec![
        set(Ptr(0), 5.0),
        set(Ptr(1), 3.0),
        set(Ptr(2), 5.0),
        set(Ptr(3), true),
        set(Ptr(4), false),
        set(Ptr(5), -0.0),
        set(Ptr(6), 0.0),
        set(Ptr(7), f64::NAN),
        set(Ptr(8), 5.0),
        // --- Number vs Number → fcmp ---
        set(Ptr(9), gt(Ptr(0), Ptr(1))),
        set(Ptr(10), lt(Ptr(0), Ptr(1))),
        set(Ptr(11), equal(Ptr(0), Ptr(2))),
        // --- Bool vs Bool → icmp ---
        set(Ptr(12), gt(Ptr(3), Ptr(4))),
        set(Ptr(13), lt(Ptr(4), Ptr(3))),
        set(Ptr(14), equal(Ptr(3), Ptr(3))),
        set(Ptr(15), equal(Ptr(4), Ptr(4))),
        // --- Number vs Bool → fcmp (bool coerced) ---
        set(Ptr(16), lt(0.5, Ptr(3))), // 0.5 < 1.0
        set(Ptr(17), lt(Ptr(4), 0.5)), // 0.0 < 0.5
        // --- Regression: -0.0 == 0.0 under regular IEEE fcmp ---
        set(Ptr(18), equal(Ptr(5), Ptr(6))), // true
        set(Ptr(19), lt(Ptr(5), Ptr(6))),    // false
        set(Ptr(20), gt(Ptr(5), Ptr(6))),    // false
        set(Ptr(21), equal(Ptr(6), Ptr(5))), // true
        set(Ptr(22), equal(Ptr(7), Ptr(7))), // true
        set(Ptr(23), gt(Ptr(7), Ptr(7))),    // false
        set(Ptr(24), lt(Ptr(7), Ptr(7))),    // false
        // --- Regression: NaN > all non-NaN in the fcmp path ---
        set(Ptr(25), gt(Ptr(7), Ptr(8))),    // true
        set(Ptr(26), lt(Ptr(7), Ptr(8))),    // false
        set(Ptr(27), equal(Ptr(7), Ptr(8))), // false
        set(Ptr(28), lt(Ptr(8), Ptr(7))),    // true
        set(Ptr(29), gt(Ptr(8), Ptr(7))),    // false
        set(Ptr(30), equal(Ptr(8), Ptr(7))), // false
    ]);

    let expected = [
        true, false, true, // num vs num
        true, true, true, true, // bool vs bool
        true, true, // num vs bool
        true, false, false, true, // -0.0 vs 0.0
        true, false, false, // NaN == NaN
        true, false, false, true, false, false, // NaN > non-NaN
    ];
    for (i, want) in expected.into_iter().enumerate() {
        assert_eq!(memory[9 + i].convert_to_bool(), want, "index {}", 9 + i);
    }
}

#[test]
fn comparison_slow_path() {
    // Slow path: at least one operand isn't provably Number/Bool, so
    // op_cmp calls the `cmp` callback (which uses the same `scratch_cmp`).
    let memory = run_code(vec![
        // Non-numeric strings
        set(Ptr(0), "apple"),
        set(Ptr(1), "banana"),
        set(Ptr(2), "apple"),
        set(Ptr(3), "Apple"),
        // Numeric strings
        set(Ptr(4), "5.0"),
        set(Ptr(5), "10.0"),
        set(Ptr(24), true), // for Bool-vs-string coercion
        set(Ptr(25), "APPLE"),
        set(Ptr(26), "Zebra"),
        set(Ptr(27), "HELLO World"),
        set(Ptr(28), "hello world"),
        // --- String vs String (byte compare) ---
        set(Ptr(6), lt(Ptr(0), Ptr(1))),
        set(Ptr(7), gt(Ptr(1), Ptr(0))),
        set(Ptr(8), equal(Ptr(0), Ptr(2))),
        set(Ptr(9), equal(Ptr(3), Ptr(0))),
        // --- Numeric strings compared as numbers ---
        set(Ptr(10), lt(Ptr(4), Ptr(5))), // 5 < 10
        // --- String literal vs heap String var ---
        set(Ptr(11), lt(Ptr(0), "banana")),
        // --- Object vs Object (uninitialised → Number(0.0) at runtime) ---
        set(Ptr(12), equal(Ptr(20), Ptr(21))),
        // --- Object vs Number ---
        set(Ptr(13), equal(Ptr(20), 0.0)),
        // --- Object vs String ("0" parses to 0.0) ---
        set(Ptr(14), equal(Ptr(20), "0")),
        // --- NaN through the slow path: force Object by mixing String + Number ---
        // Condition is falsy (Ptr(23) default-inits to Number(0.0)), so the
        // else branch runs, leaving Ptr(22) = Number(NaN) at runtime while
        // the analysis sees it as Object.
        c_if_else(
            Ptr(23),
            vec![set(Ptr(22), "hello")],
            vec![set(Ptr(22), f64::NAN)],
        ),
        set(Ptr(15), equal(Ptr(22), 5.0)), // false
        set(Ptr(16), gt(Ptr(22), 5.0)),    // true
        set(Ptr(17), lt(5.0, Ptr(22))),    // true
        // --- Regression: case-insensitive strings via the cmp callback ---
        set(Ptr(18), equal(Ptr(0), Ptr(25))), // "apple" == "APPLE"
        set(Ptr(19), gt(Ptr(26), Ptr(0))),    // "Zebra" > "apple"
        set(Ptr(29), equal(Ptr(27), Ptr(28))), // "HELLO World" == "hello world"
        set(Ptr(30), equal(Ptr(24), "TRUE")), // Bool(true) == "TRUE"
    ]);

    // String vs String
    assert_eq!(memory[6].convert_to_bool(), true);
    assert_eq!(memory[7].convert_to_bool(), true);
    assert_eq!(memory[8].convert_to_bool(), true);
    assert_eq!(memory[9].convert_to_bool(), true);
    // Numeric strings
    assert_eq!(memory[10].convert_to_bool(), true);
    // String literal
    assert_eq!(memory[11].convert_to_bool(), true);
    // Object cases (default Number(0.0))
    assert_eq!(memory[12].convert_to_bool(), true);
    assert_eq!(memory[13].convert_to_bool(), true);
    assert_eq!(memory[14].convert_to_bool(), true);
    // NaN via Object type
    assert_eq!(memory[15].convert_to_bool(), false);
    assert_eq!(memory[16].convert_to_bool(), true);
    assert_eq!(memory[17].convert_to_bool(), true);
    // Case-insensitivity
    assert_eq!(memory[18].convert_to_bool(), true);
    assert_eq!(memory[19].convert_to_bool(), true);
    assert_eq!(memory[29].convert_to_bool(), true); // "HELLO World" == "hello world"
    assert_eq!(memory[30].convert_to_bool(), true); // Bool(true) == "TRUE"
}
