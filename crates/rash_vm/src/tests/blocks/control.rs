use std::cmp::Ordering;

use crate::{
    Ptr, ScratchBlock,
    builder::{c_if, c_if_else, c_repeat, change, fadd},
    tests::blocks::{set_var, utils::run_code},
};

#[test]
pub fn nested_repeat() {
    let memory = run_code(vec![
        set_var(Ptr(0), 0.0),
        c_repeat(
            9.0,
            vec![c_repeat(
                11.0,
                vec![set_var(
                    Ptr(0),
                    ScratchBlock::OpStrJoin(Ptr(0).into(), "H".into()),
                )],
            )],
        ),
    ]);
    assert_eq!(
        memory[0].convert_to_string(),
        "0HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH"
    )
}

#[test]
pub fn repeat_until() {
    let memory = run_code(vec![
        set_var(Ptr(0), 0.0),
        ScratchBlock::ControlRepeatUntil(
            ScratchBlock::OpCmp(Ptr(0).into(), 10.0.into(), Ordering::Greater).into(),
            vec![
                set_var(Ptr(1), 0.0),
                ScratchBlock::ControlRepeatUntil(
                    ScratchBlock::OpCmp(Ptr(1).into(), 20.0.into(), Ordering::Greater).into(),
                    vec![ScratchBlock::VarChange(Ptr(1), 1.0.into())],
                ),
                ScratchBlock::VarChange(Ptr(0), 1.0.into()),
            ],
        ),
    ]);
    assert_eq!(memory[0].convert_to_number(), 11.0);
    assert_eq!(memory[1].convert_to_number(), 21.0);
}

#[test]
pub fn stop_this_script() {
    let memory = run_code(vec![
        set_var(Ptr(0), 1.0),
        set_var(Ptr(1), 2.0),
        set_var(Ptr(2), "hello"),
        ScratchBlock::ControlStopThisScript,
        set_var(Ptr(0), 999.0),
        set_var(Ptr(1), 999.0),
        set_var(Ptr(2), "bad"),
    ]);

    assert_eq!(memory[0].convert_to_number(), 1.0);
    assert_eq!(memory[1].convert_to_number(), 2.0);
    assert_eq!(memory[2].convert_to_string(), "hello");
}

#[test]
pub fn stop_this_script_inside_if_prevents_following_code() {
    let memory = run_code(vec![
        c_if(
            true,
            vec![
                set_var(Ptr(0), 10.0),
                ScratchBlock::ControlStopThisScript,
                set_var(Ptr(0), 20.0),
            ],
        ),
        set_var(Ptr(0), 30.0),
    ]);

    assert_eq!(memory[0].convert_to_number(), 10.0);
}

#[test]
pub fn stop_this_script_inside_nested_control_flow() {
    let memory = run_code(vec![
        c_if(
            true,
            vec![
                c_if(
                    true,
                    vec![
                        set_var(Ptr(0), "before stop"),
                        ScratchBlock::ControlStopThisScript,
                        set_var(Ptr(0), "after inner stop"),
                    ],
                ),
                set_var(Ptr(0), "after outer if"),
            ],
        ),
        set_var(Ptr(0), "after everything"),
    ]);

    assert_eq!(memory[0].convert_to_string(), "before stop");
}

#[test]
pub fn stop_this_script_only_skips_remaining_execution_path() {
    let memory = run_code(vec![
        set_var(Ptr(1), ""),
        c_if(
            true,
            vec![
                set_var(Ptr(0), "early"),
                ScratchBlock::ControlStopThisScript,
            ],
        ),
        set_var(Ptr(1), "should never happen"),
    ]);

    assert_eq!(memory[0].convert_to_string(), "early");
    assert_eq!(memory[1].convert_to_string(), "");
}

#[test]
pub fn stop_this_script_multiple_possible_stops() {
    let memory = run_code(vec![
        set_var(Ptr(0), "start"),
        c_if(
            true,
            vec![
                set_var(Ptr(0), "first stop"),
                ScratchBlock::ControlStopThisScript,
            ],
        ),
        set_var(Ptr(0), "second path"),
        ScratchBlock::ControlStopThisScript,
        set_var(Ptr(0), "never reached"),
    ]);

    assert_eq!(memory[0].convert_to_string(), "first stop");
}

#[test]
pub fn stop_this_script_does_not_rollback_state() {
    let memory = run_code(vec![
        set_var(Ptr(0), 100.0),
        set_var(Ptr(0), 200.0),
        ScratchBlock::ControlStopThisScript,
        set_var(Ptr(0), 300.0),
    ]);

    // The stop prevents future execution, but does not undo previous writes.
    assert_eq!(memory[0].convert_to_number(), 200.0);
}

#[test]
pub fn branch_if_else() {
    let memory = run_code(vec![
        c_if_else(true, vec![set_var(Ptr(2), 1.0)], vec![set_var(Ptr(2), 0.0)]),
        c_if_else(
            false,
            vec![set_var(Ptr(3), 0.0)],
            vec![set_var(Ptr(3), 1.0)],
        ),
        c_if_else(
            "hello",
            vec![set_var(Ptr(4), 1.0)],
            vec![set_var(Ptr(4), 0.0)],
        ),
        c_if_else("", vec![set_var(Ptr(5), 0.0)], vec![set_var(Ptr(5), 1.0)]),
        c_if_else(
            "true",
            vec![set_var(Ptr(6), 1.0)],
            vec![set_var(Ptr(6), 0.0)],
        ),
        c_if_else(
            "false",
            vec![set_var(Ptr(7), 0.0)],
            vec![set_var(Ptr(7), 1.0)],
        ),
        set_var(Ptr(0), 1.0),
        set_var(Ptr(1), 0.0),
        c_if_else(
            Ptr(0),
            vec![set_var(Ptr(8), 1.0)],
            vec![set_var(Ptr(8), 0.0)],
        ),
        c_if_else(
            Ptr(1),
            vec![set_var(Ptr(9), 0.0)],
            vec![set_var(Ptr(9), 1.0)],
        ),
    ]);
    assert_eq!(memory[0].convert_to_number(), 1.0);
    assert_eq!(memory[1].convert_to_number(), 0.0);
    assert_eq!(memory[2].convert_to_number(), 1.0);
    assert_eq!(memory[3].convert_to_number(), 1.0);
    assert_eq!(memory[4].convert_to_number(), 1.0);
    assert_eq!(memory[5].convert_to_number(), 1.0);
    assert_eq!(memory[6].convert_to_number(), 1.0);
    assert_eq!(memory[7].convert_to_number(), 1.0);
    assert_eq!(memory[8].convert_to_number(), 1.0);
    assert_eq!(memory[9].convert_to_number(), 1.0);
}

#[test]
pub fn branch_if() {
    let memory = run_code(vec![
        set_var(Ptr(0), 0.0),
        set_var(Ptr(1), 0.0),
        set_var(Ptr(2), 0.0),
        set_var(Ptr(3), 0.0),
        set_var(Ptr(4), 0.0),
        set_var(Ptr(5), 0.0),
        set_var(Ptr(6), 0.0),
        set_var(Ptr(7), 0.0),
        set_var(Ptr(8), 0.0),
        set_var(Ptr(9), 0.0),
        set_var(Ptr(10), 0.0),
        set_var(Ptr(11), 0.0),
        c_if(1.0, vec![set_var(Ptr(0), 1.0)]),
        c_if(0.0, vec![set_var(Ptr(1), 1.0)]),
        c_if(true, vec![set_var(Ptr(2), 1.0)]),
        c_if(false, vec![set_var(Ptr(3), 1.0)]),
        c_if("hello", vec![set_var(Ptr(4), 1.0)]),
        c_if("", vec![set_var(Ptr(5), 1.0)]),
        c_if("true", vec![set_var(Ptr(6), 1.0)]),
        c_if("false", vec![set_var(Ptr(7), 1.0)]),
        // nested statements
        c_if(
            true,
            vec![
                c_if(true, vec![set_var(Ptr(8), 1.0)]),
                c_if(false, vec![set_var(Ptr(9), 1.0)]),
            ],
        ),
        c_if(f64::NAN, vec![set_var(Ptr(10), 1.0)]),
        c_if(
            ScratchBlock::OpDiv(0.0.into(), 0.0.into()),
            vec![set_var(Ptr(11), 1.0)],
        ),
    ]);
    assert_eq!(memory[0].convert_to_number(), 1.0);
    assert_eq!(memory[1].convert_to_number(), 0.0);
    assert_eq!(memory[2].convert_to_number(), 1.0);
    assert_eq!(memory[3].convert_to_number(), 0.0);
    assert_eq!(memory[4].convert_to_number(), 1.0);
    assert_eq!(memory[5].convert_to_number(), 0.0);
    assert_eq!(memory[6].convert_to_number(), 1.0);
    assert_eq!(memory[7].convert_to_number(), 0.0);
    assert_eq!(memory[8].convert_to_number(), 1.0);
    assert_eq!(memory[9].convert_to_number(), 0.0);
    assert_eq!(memory[10].convert_to_number(), 0.0);
    assert_eq!(memory[11].convert_to_number(), 0.0);
}

#[test]
pub fn repeated_sum() {
    let memory = run_code(vec![
        set_var(Ptr(7), fadd(Ptr(7), false)),
        c_repeat(
            100_000.0,
            vec![
                set_var(Ptr(7), fadd(Ptr(7), true)),
                set_var(Ptr(7), fadd(Ptr(7), true)),
            ],
        ),
    ]);
    assert_eq!(memory[7].convert_to_number(), 200000.0);
}

#[test]
pub fn repeated_join_string() {
    let memory = run_code(vec![
        set_var(Ptr(7), "hello "),
        c_repeat(
            100.0,
            vec![
                set_var(
                    Ptr(7),
                    ScratchBlock::OpStrJoin(Ptr(7).into(), "world".into()),
                ),
                set_var(Ptr(7), ScratchBlock::OpStrJoin(Ptr(7).into(), ", ".into())),
            ],
        ),
    ]);
    assert_eq!(
        memory[7].convert_to_string(),
        "hello world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, world, "
    );
}

#[test]
pub fn repeat_invalid_inputs() {
    let memory = run_code(vec![
        set_var(Ptr(0), 0.0),
        c_repeat(-1.0, vec![change(Ptr(0), 1.0)]),
        set_var(Ptr(1), 0.0),
        c_repeat(0.0, vec![change(Ptr(1), 1.0)]),
        set_var(Ptr(2), 0.0),
        c_repeat(f64::NEG_INFINITY, vec![change(Ptr(2), 1.0)]),
        set_var(Ptr(3), 0.0),
        c_repeat(f64::NAN, vec![change(Ptr(3), 1.0)]),
        set_var(Ptr(4), 0.0),
        c_repeat(0.1, vec![change(Ptr(4), 1.0)]),
        set_var(Ptr(5), 0.0),
        c_repeat(0.5, vec![change(Ptr(5), 1.0)]),
        set_var(Ptr(6), 0.0),
        c_repeat(0.9, vec![change(Ptr(6), 1.0)]),
    ]);
    assert_eq!(memory[0].convert_to_number(), 0.0);
    assert_eq!(memory[1].convert_to_number(), 0.0);
    assert_eq!(memory[2].convert_to_number(), 0.0);
    assert_eq!(memory[3].convert_to_number(), 0.0);
    assert_eq!(memory[4].convert_to_number(), 0.0);
    assert_eq!(memory[5].convert_to_number(), 0.0);
    assert_eq!(memory[6].convert_to_number(), 0.0);
}

// #[test]
// pub fn repeat_invalid_inputs_runtime() {
//     let f = |n| fadd(n, 0.0);
//     let memory = run_code(vec![
//         set_var(Ptr(0), 0.0),
//         c_repeat(f(-1.0), vec![change(Ptr(0), 1.0)]),
//         set_var(Ptr(1), 0.0),
//         c_repeat(f(0.0), vec![change(Ptr(1), 1.0)]),
//         set_var(Ptr(2), 0.0),
//         c_repeat(f(f64::NEG_INFINITY), vec![change(Ptr(2), 1.0)]),
//         set_var(Ptr(3), 0.0),
//         c_repeat(f(f64::NAN), vec![change(Ptr(3), 1.0)]),
//         set_var(Ptr(4), 0.0),
//         c_repeat(f(0.1), vec![change(Ptr(4), 1.0)]),
//         set_var(Ptr(5), 0.0),
//         c_repeat(f(0.5), vec![change(Ptr(5), 1.0)]),
//         set_var(Ptr(6), 0.0),
//         c_repeat(f(0.9), vec![change(Ptr(6), 1.0)]),
//     ]);
//     assert_eq!(memory[0].convert_to_number(), 0.0);
//     assert_eq!(memory[1].convert_to_number(), 0.0);
//     assert_eq!(memory[2].convert_to_number(), 0.0);
//     assert_eq!(memory[3].convert_to_number(), 0.0);
//     assert_eq!(memory[4].convert_to_number(), 0.0);
//     assert_eq!(memory[5].convert_to_number(), 0.0);
//     assert_eq!(memory[6].convert_to_number(), 0.0);
// }
