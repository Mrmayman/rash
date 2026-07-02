mod utils;
// pub use utils::run_code;

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use utils::run_code;

    use crate::{Input, Ptr, ScratchBlock, ScratchObject, compiler::VarType};

    use super::*;

    fn set_vars(input: Vec<Input>) -> Vec<ScratchBlock> {
        input
            .into_iter()
            .enumerate()
            .map(|(i, input)| set_var(Ptr(i), input))
            .collect()
    }

    fn set_var(ptr: Ptr, input: impl Into<Input>) -> ScratchBlock {
        ScratchBlock::VarSet(ptr, input.into())
    }

    fn c_if(input: impl Into<Input>, then: Vec<ScratchBlock>) -> ScratchBlock {
        ScratchBlock::ControlIf(input.into(), then)
    }

    #[test]
    pub fn b_str_ops() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpStrJoin("hello".into(), "world".into()).into(),
            ScratchBlock::OpStrJoin("hello".into(), 1.0.into()).into(),
            ScratchBlock::OpStrJoin(1.0.into(), "world".into()).into(),
            ScratchBlock::OpStrJoin(true.into(), 2.0.into()).into(),
            ScratchBlock::OpStrLen(true.into()).into(),
            ScratchBlock::OpStrLen("💀".into()).into(),
            ScratchBlock::OpStrLetterOf(2.0.into(), "hello".into()).into(),
            ScratchBlock::OpStrLetterOf(0.0.into(), "hello".into()).into(),
            ScratchBlock::OpStrLetterOf(1.0.into(), "💀".into()).into(),
            ScratchBlock::OpStrContains("Hello World".into(), "World".into()).into(),
            ScratchBlock::OpStrContains("Hello World".into(), "world".into()).into(),
            ScratchBlock::OpStrContains("Hello World".into(), "Hi".into()).into(),
            // real bug btw
            ScratchBlock::OpStrJoin("".into(), true.into()).into(),
        ]));

        assert_eq!(memory[4].get_type(), VarType::Number);

        assert_eq!(memory[0].convert_to_string(), "helloworld");
        assert_eq!(memory[1].convert_to_string(), "hello1");
        assert_eq!(memory[2].convert_to_string(), "1world");
        assert_eq!(memory[3].convert_to_string(), "true2");
        assert_eq!(memory[4].convert_to_number(), 4.0);

        // Skull emoji takes 2 chars in Scratch.
        assert_eq!(memory[5].convert_to_number(), 2.0);

        assert_eq!(memory[6].convert_to_string(), "e");
        assert_eq!(memory[7].convert_to_string(), "");

        // memory[8] isn't valid unicode.
        // There is no way to test this.
        assert_eq!(memory[8].convert_to_string().chars().count(), 1);

        assert!(memory[9].convert_to_bool());
        assert!(memory[10].convert_to_bool());
        assert!(!memory[11].convert_to_bool());
        assert_eq!(memory[12].convert_to_string(), "true");
    }

    #[test]
    pub fn b_pi() {
        let memory = run_code(&crate::program_pi());

        assert_eq!(memory[0].convert_to_number(), -3.1415916535897743);
        assert_eq!(memory[1].convert_to_number(), 2000001.0);
        assert_eq!(memory[2].convert_to_number(), 1000000.0);
    }

    #[test]
    pub fn b_nested_repeat() {
        let memory = run_code(&vec![ScratchBlock::ControlRepeat(
            9.0.into(),
            vec![ScratchBlock::ControlRepeat(
                11.0.into(),
                vec![set_var(
                    Ptr(0),
                    ScratchBlock::OpStrJoin(Ptr(0).into(), "H".into()),
                )],
            )],
        )]);
        assert_eq!(
            memory[0].convert_to_string(),
            "0HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH"
        )
    }

    #[test]
    pub fn b_repeat_until() {
        let memory = run_code(&vec![
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

    /*
    #[test]
    pub fn b_stop_this_script() {
        let memory = run_code(&vec![
            set_var(Ptr(0), 69.0),
            ScratchBlock::ControlStopThisScript,
            set_var(Ptr(0), 420.0),
        ]);
        assert_eq!(memory[0].convert_to_number(), 69.0);
    }
    */

    #[test]
    pub fn b_if_else() {
        let memory = run_code(&vec![
            ScratchBlock::ControlIfElse(
                true.into(),
                vec![set_var(Ptr(0), 1.0)],
                vec![set_var(Ptr(0), 0.0)],
            ),
            ScratchBlock::ControlIfElse(
                false.into(),
                vec![set_var(Ptr(1), 0.0)],
                vec![set_var(Ptr(1), 1.0)],
            ),
            ScratchBlock::ControlIfElse(
                "hello".into(),
                vec![set_var(Ptr(2), 1.0)],
                vec![set_var(Ptr(2), 0.0)],
            ),
            ScratchBlock::ControlIfElse(
                String::new().into(),
                vec![set_var(Ptr(3), 0.0)],
                vec![set_var(Ptr(3), 1.0)],
            ),
            ScratchBlock::ControlIfElse(
                "true".into(),
                vec![set_var(Ptr(4), 1.0)],
                vec![set_var(Ptr(4), 0.0)],
            ),
            ScratchBlock::ControlIfElse(
                "false".into(),
                vec![set_var(Ptr(5), 0.0)],
                vec![set_var(Ptr(5), 1.0)],
            ),
        ]);
        assert_eq!(memory[0].convert_to_number(), 1.0);
        assert_eq!(memory[1].convert_to_number(), 1.0);
        assert_eq!(memory[2].convert_to_number(), 1.0);
        assert_eq!(memory[3].convert_to_number(), 1.0);
        assert_eq!(memory[4].convert_to_number(), 1.0);
        assert_eq!(memory[5].convert_to_number(), 1.0);
    }

    #[test]
    pub fn b_if() {
        let memory = run_code(&vec![
            c_if(1.0, vec![set_var(Ptr(0), 1.0)]),
            c_if(0.0, vec![set_var(Ptr(1), 1.0)]),
            c_if(true, vec![set_var(Ptr(2), 1.0)]),
            c_if(false, vec![set_var(Ptr(3), 1.0)]),
            c_if("hello", vec![set_var(Ptr(4), 1.0)]),
            c_if(String::new(), vec![set_var(Ptr(5), 1.0)]),
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
    pub fn b_repeated_sum() {
        let memory = run_code(&vec![
            set_var(Ptr(7), ScratchBlock::OpAdd(Ptr(7).into(), false.into())),
            ScratchBlock::ControlRepeat(
                100_000.0.into(),
                vec![
                    set_var(Ptr(7), ScratchBlock::OpAdd(Ptr(7).into(), true.into())),
                    set_var(Ptr(7), ScratchBlock::OpAdd(Ptr(7).into(), true.into())),
                ],
            ),
        ]);
        assert_eq!(memory[7].convert_to_number(), 200000.0);
    }

    #[test]
    pub fn b_repeated_join_string() {
        let memory = run_code(&vec![
            set_var(Ptr(7), "hello "),
            ScratchBlock::ControlRepeat(
                100.0.into(),
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
    pub fn b_random() {
        let memory = run_code(&vec![
            set_var(Ptr(0), ScratchBlock::OpRandom(0.0.into(), 100.0.into())),
            set_var(Ptr(1), ScratchBlock::OpRandom(1.0.into(), 2.5.into())),
            set_var(Ptr(2), ScratchBlock::OpRandom("1".into(), "2".into())),
            set_var(Ptr(3), ScratchBlock::OpRandom("1.0".into(), "2".into())),
            ScratchBlock::ControlRepeat(
                100_000.0.into(),
                vec![set_var(
                    Ptr(4),
                    ScratchBlock::OpRandom(0.0.into(), 100.0.into()),
                )],
            ),
        ]);

        assert!(memory[0].convert_to_number() >= 0.0);
        assert!(memory[0].convert_to_number() <= 100.0);
        assert_eq!(memory[0].convert_to_number().fract(), 0.0);

        assert!(memory[1].convert_to_number() >= 1.0);
        assert!(memory[1].convert_to_number() <= 2.5);
        assert_ne!(memory[1].convert_to_number().fract(), 0.0);

        assert!(memory[2].convert_to_number() >= 1.0);
        assert!(memory[2].convert_to_number() <= 2.0);
        assert_eq!(memory[2].convert_to_number().fract(), 0.0);

        assert!(memory[3].convert_to_number() >= 1.0);
        assert!(memory[3].convert_to_number() <= 2.0);
        assert_ne!(memory[3].convert_to_number().fract(), 0.0);

        assert!(memory[4].convert_to_number() >= 0.0);
        assert!(memory[4].convert_to_number() <= 100.0);
        assert_eq!(memory[4].convert_to_number().fract(), 0.0);
    }

    #[test]
    pub fn b_math_add() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpAdd(50.0.into(), 25.0.into()).into(),
            ScratchBlock::OpAdd((-500.0).into(), 25.0.into()).into(),
            ScratchBlock::OpAdd((-500.0).into(), (-25.0).into()).into(),
            ScratchBlock::OpAdd(2.54.into(), 6.25.into()).into(),
            ScratchBlock::OpAdd(2.54.into(), (-6.25).into()).into(),
            ScratchBlock::OpAdd(true.into(), true.into()).into(),
            ScratchBlock::OpAdd((1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
            ScratchBlock::OpAdd((1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
            ScratchBlock::OpAdd((-1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
            ScratchBlock::OpAdd((-1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
            ScratchBlock::OpAdd(1.0.into(), f64::NAN.into()).into(),
            ScratchBlock::OpAdd(f64::NAN.into(), 1.0.into()).into(),
        ]));
        assert_eq!(memory[0].convert_to_number(), 75.0);
        assert_eq!(memory[1].convert_to_number(), -475.0);
        assert_eq!(memory[2].convert_to_number(), -525.0);
        assert_eq!(memory[3].convert_to_number(), 8.79);
        assert_eq!(memory[4].convert_to_number(), -3.71);
        assert_eq!(memory[5].convert_to_number(), 2.0);

        assert!(memory[6].convert_to_number().is_infinite());
        assert!(memory[7].convert_to_number().is_nan());
        assert!(memory[8].convert_to_number().is_nan());
        assert!(memory[9].convert_to_number().is_infinite());
        assert!(memory[9].convert_to_number().is_sign_negative());

        assert_eq!(memory[10].convert_to_number(), 1.0);
        assert_eq!(memory[11].convert_to_number(), 1.0);
    }

    #[test]
    pub fn b_math_sub() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpSub(50.0.into(), 25.0.into()).into(),
            ScratchBlock::OpSub((-500.0).into(), 25.0.into()).into(),
            ScratchBlock::OpSub((-500.0).into(), (-25.0).into()).into(),
            ScratchBlock::OpSub(2.54.into(), 6.25.into()).into(),
            ScratchBlock::OpSub(2.54.into(), (-6.25).into()).into(),
            ScratchBlock::OpSub(true.into(), true.into()).into(),
            ScratchBlock::OpSub((1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
            ScratchBlock::OpSub((1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
            ScratchBlock::OpSub((-1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
            ScratchBlock::OpSub((-1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
            ScratchBlock::OpSub(1.0.into(), f64::NAN.into()).into(),
            ScratchBlock::OpSub(f64::NAN.into(), 1.0.into()).into(),
        ]));
        assert_eq!(memory[0].convert_to_number(), 25.0);
        assert_eq!(memory[1].convert_to_number(), -525.0);
        assert_eq!(memory[2].convert_to_number(), -475.0);
        assert_eq!(memory[3].convert_to_number(), -3.71);
        assert_eq!(memory[4].convert_to_number(), 8.79);
        assert_eq!(memory[5].convert_to_number(), 0.0);

        assert!(memory[6].convert_to_number().is_nan());
        assert!(memory[7].convert_to_number().is_infinite());
        assert!(memory[7].convert_to_number().is_sign_positive());
        assert!(memory[8].convert_to_number().is_infinite());
        assert!(memory[8].convert_to_number().is_sign_negative());
        assert!(memory[9].convert_to_number().is_nan());

        assert_eq!(memory[10].convert_to_number(), 1.0);
        assert_eq!(memory[11].convert_to_number(), -1.0);
    }

    #[test]
    pub fn b_math_mul() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpMul(50.0.into(), 2.0.into()).into(),
            ScratchBlock::OpMul((-50.0).into(), 2.0.into()).into(),
            ScratchBlock::OpMul((-50.0).into(), (-2.0).into()).into(),
            ScratchBlock::OpMul(2.54.into(), 6.25.into()).into(),
            ScratchBlock::OpMul(2.54.into(), (-6.25).into()).into(),
            ScratchBlock::OpMul(true.into(), true.into()).into(),
            ScratchBlock::OpMul((1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
            ScratchBlock::OpMul((1.0 / 0.0).into(), 0.0.into()).into(),
            ScratchBlock::OpMul((1.0 / 0.0).into(), 2.0.into()).into(),
            ScratchBlock::OpMul((1.0 / 0.0).into(), (-2.0).into()).into(),
            ScratchBlock::OpMul((1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), 0.0.into()).into(),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), 2.0.into()).into(),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), (-2.0).into()).into(),
            ScratchBlock::OpMul((-1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
            ScratchBlock::OpMul(1.0.into(), f64::NAN.into()).into(),
            ScratchBlock::OpMul(f64::NAN.into(), 1.0.into()).into(),
        ]));
        assert_eq!(memory[0].convert_to_number(), 100.0);
        assert_eq!(memory[1].convert_to_number(), -100.0);
        assert_eq!(memory[2].convert_to_number(), 100.0);
        assert_eq!(memory[3].convert_to_number(), 15.875);
        assert_eq!(memory[4].convert_to_number(), -15.875);
        assert_eq!(memory[5].convert_to_number(), 1.0);

        assert!(memory[6].convert_to_number().is_infinite());
        assert!(memory[6].convert_to_number().is_sign_positive());

        assert!(memory[7].convert_to_number().is_nan());

        assert!(memory[8].convert_to_number().is_infinite());
        assert!(memory[8].convert_to_number().is_sign_positive());

        assert!(memory[9].convert_to_number().is_infinite());
        assert!(memory[9].convert_to_number().is_sign_negative());

        assert!(memory[10].convert_to_number().is_infinite());
        assert!(memory[10].convert_to_number().is_sign_negative());

        assert!(memory[11].convert_to_number().is_infinite());
        assert!(memory[11].convert_to_number().is_sign_negative());

        assert!(memory[12].convert_to_number().is_nan());

        assert!(memory[13].convert_to_number().is_infinite());
        assert!(memory[13].convert_to_number().is_sign_negative());

        assert!(memory[14].convert_to_number().is_infinite());
        assert!(memory[14].convert_to_number().is_sign_positive());

        assert!(memory[15].convert_to_number().is_infinite());
        assert!(memory[15].convert_to_number().is_sign_positive());

        assert_eq!(memory[16].convert_to_number(), 0.0);
        assert_eq!(memory[17].convert_to_number(), 0.0);
    }

    #[test]
    pub fn b_math_div() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpDiv(50.0.into(), 2.0.into()).into(),
            ScratchBlock::OpDiv((-50.0).into(), 2.0.into()).into(),
            ScratchBlock::OpDiv((-50.0).into(), (-2.0).into()).into(),
            ScratchBlock::OpDiv(3.5.into(), 2.5.into()).into(),
            ScratchBlock::OpDiv(3.5.into(), (-2.5).into()).into(),
            ScratchBlock::OpDiv(true.into(), true.into()).into(),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), 0.0.into()).into(),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), 2.0.into()).into(),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), (-2.0).into()).into(),
            ScratchBlock::OpDiv((1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), (1.0 / 0.0).into()).into(),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), 0.0.into()).into(),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), 2.0.into()).into(),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), (-2.0).into()).into(),
            ScratchBlock::OpDiv((-1.0 / 0.0).into(), (-1.0 / 0.0).into()).into(),
            ScratchBlock::OpDiv(1.0.into(), f64::NAN.into()).into(),
            ScratchBlock::OpDiv(f64::NAN.into(), 1.0.into()).into(),
        ]));
        assert_eq!(memory[0].convert_to_number(), 25.0);
        assert_eq!(memory[1].convert_to_number(), -25.0);
        assert_eq!(memory[2].convert_to_number(), 25.0);
        assert_eq!(memory[3].convert_to_number(), 1.4);
        assert_eq!(memory[4].convert_to_number(), -1.4);
        assert_eq!(memory[5].convert_to_number(), 1.0);

        assert!(memory[6].convert_to_number().is_nan());

        assert!(memory[7].convert_to_number().is_infinite());
        assert!(memory[7].convert_to_number().is_sign_positive());

        assert!(memory[8].convert_to_number().is_infinite());
        assert!(memory[8].convert_to_number().is_sign_positive());

        assert!(memory[9].convert_to_number().is_infinite());
        assert!(memory[9].convert_to_number().is_sign_negative());

        assert!(memory[10].convert_to_number().is_nan());
        assert!(memory[11].convert_to_number().is_nan());

        assert!(memory[12].convert_to_number().is_infinite());
        assert!(memory[12].convert_to_number().is_sign_negative());

        assert!(memory[13].convert_to_number().is_infinite());
        assert!(memory[13].convert_to_number().is_sign_negative());

        assert!(memory[14].convert_to_number().is_infinite());
        assert!(memory[14].convert_to_number().is_sign_positive());

        assert!(memory[15].convert_to_number().is_nan());

        assert!(memory[16].convert_to_number().is_infinite());
        assert!(memory[16].convert_to_number().is_sign_positive());

        assert_eq!(memory[17].convert_to_number(), 0.0);
    }

    #[test]
    pub fn b_bool_ops() {
        fn check_and(memory: &[ScratchObject], offset: usize) {
            assert_eq!(memory[offset + 0].convert_to_number(), 1.0);
            assert_eq!(memory[offset + 1].convert_to_number(), 0.0);
            assert_eq!(memory[offset + 2].convert_to_number(), 0.0);
            assert_eq!(memory[offset + 3].convert_to_number(), 0.0);
        }

        fn check_or(memory: &[ScratchObject], offset: usize) {
            assert_eq!(memory[offset + 0].convert_to_number(), 1.0);
            assert_eq!(memory[offset + 1].convert_to_number(), 1.0);
            assert_eq!(memory[offset + 2].convert_to_number(), 1.0);
            assert_eq!(memory[offset + 3].convert_to_number(), 0.0);
        }

        // Testing multiple data types as this caused a real bug earlier
        let memory = run_code(&set_vars(vec![
            // Bools
            ScratchBlock::OpBAnd(true.into(), true.into()).into(),
            ScratchBlock::OpBAnd(true.into(), false.into()).into(),
            ScratchBlock::OpBAnd(false.into(), true.into()).into(),
            ScratchBlock::OpBAnd(false.into(), false.into()).into(),
            ScratchBlock::OpBOr(true.into(), true.into()).into(),
            ScratchBlock::OpBOr(true.into(), false.into()).into(),
            ScratchBlock::OpBOr(false.into(), true.into()).into(),
            ScratchBlock::OpBOr(false.into(), false.into()).into(),
            ScratchBlock::OpBNot(true.into()).into(),
            ScratchBlock::OpBNot(false.into()).into(),
            // Numbers
            ScratchBlock::OpBNot(1.0.into()).into(),
            ScratchBlock::OpBNot(0.0.into()).into(),
            ScratchBlock::OpBAnd(1.0.into(), 1.0.into()).into(),
            ScratchBlock::OpBAnd(1.0.into(), 0.0.into()).into(),
            ScratchBlock::OpBAnd(0.0.into(), 1.0.into()).into(),
            ScratchBlock::OpBAnd(0.0.into(), 0.0.into()).into(),
            ScratchBlock::OpBOr(1.0.into(), 1.0.into()).into(),
            ScratchBlock::OpBOr(1.0.into(), 0.0.into()).into(),
            ScratchBlock::OpBOr(0.0.into(), 1.0.into()).into(),
            ScratchBlock::OpBOr(0.0.into(), 0.0.into()).into(),
            // Strings
            ScratchBlock::OpBAnd("true".into(), "true".into()).into(),
            ScratchBlock::OpBAnd("true".into(), "false".into()).into(),
            ScratchBlock::OpBAnd("false".into(), "true".into()).into(),
            ScratchBlock::OpBAnd("false".into(), "false".into()).into(),
            ScratchBlock::OpBOr("true".into(), "true".into()).into(),
            ScratchBlock::OpBOr("true".into(), "false".into()).into(),
            ScratchBlock::OpBOr("false".into(), "true".into()).into(),
            ScratchBlock::OpBOr("false".into(), "false".into()).into(),
            ScratchBlock::OpBNot("true".into()).into(),
            ScratchBlock::OpBNot("false".into()).into(),
        ]));

        check_and(&memory, 0);
        check_or(&memory, 4);

        assert_eq!(memory[8].convert_to_number(), 0.0);
        assert_eq!(memory[9].convert_to_number(), 1.0);
        assert_eq!(memory[10].convert_to_number(), 0.0);
        assert_eq!(memory[11].convert_to_number(), 1.0);

        check_and(&memory, 12);
        check_or(&memory, 16);
        check_and(&memory, 20);
        check_or(&memory, 24);

        assert_eq!(memory[28].convert_to_number(), 0.0);
        assert_eq!(memory[29].convert_to_number(), 1.0);
    }

    #[test]
    pub fn b_math_modulo() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpMod(5.5.into(), 3.0.into()).into(), // 5.5 % 3.0
            ScratchBlock::OpMod((-5.5).into(), 3.0.into()).into(), // -5.5 % 3.0
            ScratchBlock::OpMod(5.5.into(), (-3.0).into()).into(), // 5.5 % -3.0
            ScratchBlock::OpMod((-5.5).into(), (-3.0).into()).into(), // -5.5 % -3.0
            ScratchBlock::OpMod(10.0.into(), 3.0.into()).into(), // 10.0 % 3.0
            ScratchBlock::OpMod((-10.0).into(), 3.0.into()).into(), // -10.0 % 3.0
            ScratchBlock::OpMod(10.0.into(), (-3.0).into()).into(), // 10.0 % -3.0
            ScratchBlock::OpMod((-10.0).into(), (-3.0).into()).into(), // -10.0 % -3.0
            ScratchBlock::OpMod(0.0.into(), 1.0.into()).into(), // 0.0 % 1.0
            ScratchBlock::OpMod((-1.0).into(), 1.0.into()).into(), // -1.0 % 1.0
            ScratchBlock::OpMod(1.0.into(), (-1.0).into()).into(), // 1.0 % -1.0
            ScratchBlock::OpMod(1.0.into(), 2.5.into()).into(), // 1.0 % 2.5
            ScratchBlock::OpMod((-1.0).into(), 2.5.into()).into(), // -1.0 % 2.5
            ScratchBlock::OpMod(1e10.into(), 3.0.into()).into(), // Large numbers
            ScratchBlock::OpMod((-1e10).into(), 3.0.into()).into(),
            ScratchBlock::OpMod(0.0001.into(), 0.003.into()).into(), // Small remainders
            ScratchBlock::OpMod((-0.0001).into(), 0.003.into()).into(),
        ]));

        assert_eq!(memory[0].convert_to_number(), 2.5);
        assert!((memory[1].convert_to_number() - 0.5) <= f64::EPSILON);
        assert!((memory[2].convert_to_number() + 0.5).abs() <= f64::EPSILON);
        assert_eq!(memory[3].convert_to_number(), -2.5);

        assert!((memory[4].convert_to_number() - 1.0) <= 2.0 * f64::EPSILON);
        assert!((memory[5].convert_to_number() - 2.0).abs() <= 2.0 * f64::EPSILON);
        assert!((memory[6].convert_to_number() + 2.0).abs() <= 2.0 * f64::EPSILON);
        assert!((memory[7].convert_to_number() + 1.0).abs() <= 2.0 * f64::EPSILON);

        assert_eq!(memory[8].convert_to_number(), 0.0);
        assert_eq!(memory[9].convert_to_number(), 0.0);
        assert_eq!(memory[10].convert_to_number(), 0.0);

        assert_eq!(memory[11].convert_to_number(), 1.0);
        assert_eq!(memory[12].convert_to_number(), 1.5);

        assert!(
            (memory[13].convert_to_number() - 1.0).abs() <= (i32::MAX as f64 + 1.0) * f64::EPSILON
        );
        assert!(
            (memory[14].convert_to_number() - 2.0).abs() <= (i32::MAX as f64 + 1.0) * f64::EPSILON
        );

        assert_eq!(memory[15].convert_to_number(), 0.0001);
        assert!(memory[16].convert_to_number() - 0.0029 <= f64::EPSILON);
    }

    #[test]
    pub fn b_math_floor() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpMFloor(5.5.into()).into(),
            ScratchBlock::OpMFloor((-3.2).into()).into(),
            ScratchBlock::OpMFloor(0.0.into()).into(),
            ScratchBlock::OpMFloor((-0.8).into()).into(),
            ScratchBlock::OpMFloor(2.999.into()).into(),
            ScratchBlock::OpMFloor((-1.1).into()).into(),
            ScratchBlock::OpMFloor(10.0.into()).into(),
            ScratchBlock::OpMFloor((-10.999).into()).into(),
            ScratchBlock::OpMFloor(123_456.789.into()).into(),
            ScratchBlock::OpMFloor((-123_456.789).into()).into(),
            ScratchBlock::OpMFloor(1e-9.into()).into(),
            ScratchBlock::OpMFloor((-1e-9).into()).into(),
            ScratchBlock::OpMFloor(1e10.into()).into(),
            ScratchBlock::OpMFloor((-1e10).into()).into(),
        ]));
        assert_eq!(memory[0].convert_to_number(), 5.0);
        assert_eq!(memory[1].convert_to_number(), -4.0);
        assert_eq!(memory[2].convert_to_number(), 0.0);
        assert_eq!(memory[3].convert_to_number(), -1.0);
        assert_eq!(memory[4].convert_to_number(), 2.0);
        assert_eq!(memory[5].convert_to_number(), -2.0);
        assert_eq!(memory[6].convert_to_number(), 10.0);
        assert_eq!(memory[7].convert_to_number(), -11.0);
        assert_eq!(memory[8].convert_to_number(), 123456.0);
        assert_eq!(memory[9].convert_to_number(), -123457.0);
        assert_eq!(memory[10].convert_to_number(), 0.0);
        assert_eq!(memory[11].convert_to_number(), -1.0);
        assert_eq!(memory[12].convert_to_number(), 1e10);
        assert_eq!(memory[13].convert_to_number(), -1e10);
    }

    #[test]
    pub fn b_math_round() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpRound(2.3.into()).into(),
            ScratchBlock::OpRound(2.5.into()).into(),
            ScratchBlock::OpRound(2.7.into()).into(),
            ScratchBlock::OpRound(3.0.into()).into(),
            ScratchBlock::OpRound((-2.3).into()).into(),
            ScratchBlock::OpRound((-2.5).into()).into(),
            ScratchBlock::OpRound((-2.7).into()).into(),
            ScratchBlock::OpRound((-3.0).into()).into(),
        ]));
        assert_eq!(memory[0].convert_to_number(), 2.0);
        assert_eq!(memory[1].convert_to_number(), 3.0);
        assert_eq!(memory[2].convert_to_number(), 3.0);
        assert_eq!(memory[3].convert_to_number(), 3.0);
        assert_eq!(memory[4].convert_to_number(), -2.0);
        assert_eq!(memory[5].convert_to_number(), -2.0);
        assert_eq!(memory[6].convert_to_number(), -3.0);
        assert_eq!(memory[7].convert_to_number(), -3.0);
    }

    #[test]
    pub fn b_math_abs() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpMAbs(2.3.into()).into(),
            ScratchBlock::OpMAbs((-2.3).into()).into(),
            ScratchBlock::OpMAbs(0.0.into()).into(),
            ScratchBlock::OpMAbs((-0.0).into()).into(),
            ScratchBlock::OpMAbs(f64::INFINITY.into()).into(),
            ScratchBlock::OpMAbs(f64::NEG_INFINITY.into()).into(),
        ]));
        assert_eq!(memory[0].convert_to_number(), 2.3);
        assert_eq!(memory[1].convert_to_number(), 2.3);
        assert_eq!(memory[2].convert_to_number(), 0.0);
        assert_eq!(memory[3].convert_to_number(), 0.0);
        assert_eq!(memory[4].convert_to_number(), f64::INFINITY);
        assert_eq!(memory[5].convert_to_number(), f64::INFINITY);
    }

    #[test]
    pub fn b_math_sqrt() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpMSqrt(1.0.into()).into(),
            ScratchBlock::OpMSqrt(2.0.into()).into(),
            ScratchBlock::OpMSqrt(0.0.into()).into(),
            ScratchBlock::OpMSqrt((-0.0).into()).into(),
            ScratchBlock::OpMSqrt((-1.0).into()).into(),
            ScratchBlock::OpMSqrt(f64::INFINITY.into()).into(),
            ScratchBlock::OpMSqrt(f64::NEG_INFINITY.into()).into(),
        ]));
        assert_eq!(memory[0].convert_to_number(), 1.0);
        assert_eq!(memory[1].convert_to_number(), 1.4142135623730951);
        assert_eq!(memory[2].convert_to_number(), 0.0);
        assert_eq!(memory[3].convert_to_number(), 0.0);
        assert!(memory[4].convert_to_number().is_nan());
        assert!(memory[5].convert_to_number().is_infinite());
        assert!(memory[5].convert_to_number().is_sign_positive());
        assert!(memory[6].convert_to_number().is_nan());
    }

    #[test]
    pub fn b_math_trig() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpMSin(0.0.into()).into(),
            ScratchBlock::OpMSin(30.0.into()).into(),
            ScratchBlock::OpMSin(60.0.into()).into(),
            ScratchBlock::OpMSin(90.0.into()).into(),
            ScratchBlock::OpMSin(100.0.into()).into(),
            ScratchBlock::OpMSin((-30.0).into()).into(),
            ScratchBlock::OpMSin((-60.0).into()).into(),
            ScratchBlock::OpMSin((-90.0).into()).into(),
            ScratchBlock::OpMSin((-100.0).into()).into(),
            ScratchBlock::OpMCos(0.0.into()).into(),
            ScratchBlock::OpMCos(30.0.into()).into(),
            ScratchBlock::OpMCos(60.0.into()).into(),
            ScratchBlock::OpMCos(90.0.into()).into(),
            ScratchBlock::OpMCos(100.0.into()).into(),
            ScratchBlock::OpMCos((-30.0).into()).into(),
            ScratchBlock::OpMCos((-60.0).into()).into(),
            ScratchBlock::OpMCos((-90.0).into()).into(),
            ScratchBlock::OpMCos((-100.0).into()).into(),
            ScratchBlock::OpMTan(0.0.into()).into(),
            ScratchBlock::OpMTan(30.0.into()).into(),
            ScratchBlock::OpMTan(60.0.into()).into(),
            ScratchBlock::OpMTan(90.0.into()).into(),
            ScratchBlock::OpMTan(100.0.into()).into(),
            ScratchBlock::OpMTan(180.0.into()).into(),
            ScratchBlock::OpMTan(270.0.into()).into(),
            ScratchBlock::OpMTan((-30.0).into()).into(),
            ScratchBlock::OpMTan((-60.0).into()).into(),
            ScratchBlock::OpMTan((-90.0).into()).into(),
            ScratchBlock::OpMTan((-100.0).into()).into(),
            ScratchBlock::OpMTan((-180.0).into()).into(),
            ScratchBlock::OpMTan((-270.0).into()).into(),
        ]));

        // SIN

        assert_eq!(memory[0].convert_to_number(), 0.0);

        assert!(0.5 - memory[1].convert_to_number() < f64::EPSILON);
        assert!(0.8660254038 - memory[2].convert_to_number() <= 80000.0 * f64::EPSILON);
        assert_eq!(memory[3].convert_to_number(), 1.0);
        assert!(memory[4].convert_to_number() - 0.984807753 <= 60000.0 * f64::EPSILON);

        assert!(memory[5].convert_to_number() - 0.5 < f64::EPSILON);
        assert!(memory[6].convert_to_number() - 0.8660254038 <= 80000.0 * f64::EPSILON);
        assert_eq!(memory[7].convert_to_number(), -1.0);
        assert!(memory[8].convert_to_number() + 0.984807753 <= 60000.0 * f64::EPSILON);

        // COS

        assert_eq!(memory[9].convert_to_number(), 1.0);

        assert!(0.8660254038 - memory[10].convert_to_number() <= 80000.0 * f64::EPSILON);
        assert!(0.5 - memory[11].convert_to_number() < f64::EPSILON);
        assert!(memory[12].convert_to_number() < f64::EPSILON);
        assert!(memory[13].convert_to_number() + 0.1736481777 < 150000.0 * f64::EPSILON);

        assert!(memory[14].convert_to_number() - 0.8660254038 <= 80000.0 * f64::EPSILON);
        assert!(memory[15].convert_to_number() - 0.5 < f64::EPSILON);
        assert!(memory[16].convert_to_number() < f64::EPSILON);
        assert!(memory[17].convert_to_number() + 0.1736481777 < 150000.0 * f64::EPSILON);

        // TAN

        assert_eq!(memory[18].convert_to_number(), 0.0);

        assert!((memory[19].convert_to_number() - 0.5773502692).abs() < 46722.0 * f64::EPSILON);
        assert!((memory[20].convert_to_number() - 1.7320508076).abs() < 140168.0 * f64::EPSILON);
        assert!(memory[21].convert_to_number().is_infinite());
        assert!(memory[21].convert_to_number().is_sign_positive());
        assert!(memory[22].convert_to_number() - 5.6712818196 < 79765.0 * f64::EPSILON);
        assert!(memory[23].convert_to_number().abs() < f64::EPSILON);
        assert!(memory[24].convert_to_number().is_infinite());
        assert!(memory[24].convert_to_number().is_sign_negative());

        assert!((memory[25].convert_to_number() + 0.5773502692).abs() < 46722.0 * f64::EPSILON);
        assert!((memory[26].convert_to_number() + 1.7320508076).abs() < 140168.0 * f64::EPSILON);
        assert!(memory[27].convert_to_number().is_infinite());
        assert!(memory[27].convert_to_number().is_sign_negative());
        assert!(memory[28].convert_to_number() - 5.6712818196 < 79765.0 * f64::EPSILON);
        assert!(memory[29].convert_to_number() < f64::EPSILON);
        assert!(memory[30].convert_to_number().is_infinite());
        assert!(memory[30].convert_to_number().is_sign_positive());
    }

    #[test]
    fn b_bool_return() {
        let memory = run_code(&vec![set_var(
            Ptr(0),
            ScratchBlock::OpBAnd(
                ScratchBlock::OpCmp(3.0.into(), 2.0.into(), Ordering::Greater).into(),
                false.into(),
            ),
        )]);
        assert_eq!(memory[0].convert_to_number(), 0.0);
    }

    #[test]
    fn b_comparison() {
        let memory = run_code(&set_vars(vec![
            ScratchBlock::OpCmp(3.0.into(), 2.0.into(), Ordering::Greater).into(),
            ScratchBlock::OpCmp(2.0.into(), 3.0.into(), Ordering::Greater).into(),
            ScratchBlock::OpCmp(3.0.into(), 3.0.into(), Ordering::Greater).into(),
            ScratchBlock::OpCmp(3.0.into(), 2.0.into(), Ordering::Less).into(),
            ScratchBlock::OpCmp(2.0.into(), 3.0.into(), Ordering::Less).into(),
            ScratchBlock::OpCmp(3.0.into(), 3.0.into(), Ordering::Less).into(),
        ]));
        assert_eq!(memory[0].convert_to_number(), 1.0);
        assert_eq!(memory[1].convert_to_number(), 0.0);
        assert_eq!(memory[2].convert_to_number(), 0.0);
        assert_eq!(memory[3].convert_to_number(), 0.0);
        assert_eq!(memory[4].convert_to_number(), 1.0);
        assert_eq!(memory[5].convert_to_number(), 0.0);
    }
}
