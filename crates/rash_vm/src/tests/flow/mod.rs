// Note: Scratch loops omit a screen refresh
// if there aren't any graphics/time related
// operations inside.

// But here, we need to test screen refresh.
// However we can't use graphical operations
// inside a test environment, so we just use
// an explicit ScreenRefresh

use crate::{
    ScratchObject,
    builder::{c_repeat, change, set},
    compiler::ScratchBlock,
    input_primitives::Ptr,
    runtime::{CustomBlockId, ProjectBuilder, Script, SpriteBuilder},
};
use rash_core::RunState;

mod args;

const X: Ptr = Ptr(0);

fn suite(scripts: Vec<Script>, num_ticks_expected: usize) -> Box<[ScratchObject]> {
    fn by_two(n: usize) -> usize {
        if n % 2 == 0 {
            // If n is even, standard division works
            n / 2
        } else {
            // If n is odd, integer division truncates, so add 1
            n / 2 + 1
        }
    }

    let mut builder = ProjectBuilder::new();
    let mut sprite1 = SpriteBuilder::new();

    for script in scripts {
        sprite1.add_script(script);
    }
    builder.add_sprite(sprite1);

    let mut runtime = builder.build();

    let mut num_ticks = 1;
    let mut graphics = RunState::default();
    while !runtime.update(&mut graphics) {
        num_ticks += 1;
    }

    assert_eq!(by_two(num_ticks), num_ticks_expected);
    runtime.memory
}

#[test]
fn custom_block_screen_refresh() {
    let m = suite(
        vec![
            Script::new_custom_block(
                vec![c_repeat(
                    3.0,
                    vec![
                        change(X, ScratchBlock::FunctionGetArg(0)),
                        change(X, ScratchBlock::FunctionGetArg(1)),
                        ScratchBlock::ScreenRefresh,
                    ],
                )],
                2,
                CustomBlockId(1),
                true,
            ),
            Script::new_custom_block(
                vec![
                    set(X, 0.0),
                    c_repeat(
                        5.0,
                        vec![
                            ScratchBlock::FunctionCallScreenRefresh(
                                CustomBlockId(1),
                                vec![1.0.into(), 2.0.into()],
                            ),
                            ScratchBlock::ScreenRefresh,
                        ],
                    ),
                ],
                0,
                CustomBlockId(0),
                true,
            ),
            Script::new_green_flag(vec![
                set(X, 0.0),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), Vec::new()),
            ]),
        ],
        21,
    );
    assert_eq!(m[0].convert_to_number(), 45.0);
}

#[test]
fn nested_loop_screen_refresh() {
    let m = suite(
        vec![Script::new_green_flag(vec![
            set(X, 0.5),
            c_repeat(
                3.0,
                vec![
                    c_repeat(4.0, vec![change(X, 2.0), ScratchBlock::ScreenRefresh]),
                    ScratchBlock::ScreenRefresh,
                ],
            ),
        ])],
        16,
    );
    assert_eq!(m[0].convert_to_number(), 24.5);
}

#[test]
fn warp_custom_block_does_not_refresh() {
    // ScreenRefresh inside warp block should still force yielding
    let m = suite(
        vec![
            Script::new_custom_block(
                vec![change(X, 1.0), ScratchBlock::ScreenRefresh, change(X, 1.0)],
                0,
                CustomBlockId(0),
                true, // warp
            ),
            Script::new_green_flag(vec![
                set(X, 0.0),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![]),
            ]),
        ],
        2,
    );
    assert_eq!(m[0].convert_to_number(), 2.0)
}

#[test]
fn no_refresh_blocks_complete_same_tick() {
    // Everything should happen before the next frame
    let mut blocks = vec![set(X, 0.0)];
    blocks.extend((0..100).map(|_| change(X, 1.0)));
    let m = suite(vec![Script::new_green_flag(blocks)], 1);
    assert_eq!(m[0].convert_to_number(), 100.0);
}

#[test]
fn nested_custom_blocks_preserve_refresh_order() {
    let m = suite(
        vec![
            // Inner block
            Script::new_custom_block(
                vec![change(X, 1.0), ScratchBlock::ScreenRefresh],
                0,
                CustomBlockId(1),
                true,
            ),
            // Outer block
            Script::new_custom_block(
                vec![
                    ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(1), vec![]),
                    change(X, 10.0),
                    ScratchBlock::ScreenRefresh,
                ],
                0,
                CustomBlockId(0),
                true,
            ),
            Script::new_green_flag(vec![
                set(X, 0.0),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![]),
            ]),
        ],
        3,
    );
    assert_eq!(m[0].convert_to_number(), 11.0);
}

#[test]
fn multiple_scripts_refresh_independently() {
    let m = suite(
        vec![
            Script::new_green_flag(vec![
                set(X, 0.0),
                c_repeat(3.0, vec![change(X, 1.0), ScratchBlock::ScreenRefresh]),
            ]),
            Script::new_green_flag(vec![
                set(Ptr(1), 0.0),
                c_repeat(3.0, vec![change(Ptr(1), 1.0), ScratchBlock::ScreenRefresh]),
            ]),
        ],
        4,
    );
    assert_eq!(m[0].convert_to_number(), 3.0);
    assert_eq!(m[1].convert_to_number(), 3.0);
}

#[test]
fn repeat_zero_and_negative_do_not_execute() {
    let m = suite(
        vec![Script::new_green_flag(vec![
            set(X, 0.0),
            c_repeat(0.0, vec![change(X, 1.0)]),
            c_repeat(-5.0, vec![change(X, 1.0)]),
        ])],
        1,
    );
    assert_eq!(m[0].convert_to_number(), 0.0);
}

#[test]
fn refresh_after_loop_happens_after_last_iteration() {
    let m = suite(
        vec![Script::new_green_flag(vec![
            set(X, 0.0),
            c_repeat(2.0, vec![change(X, 1.0), ScratchBlock::ScreenRefresh]),
            change(X, 10.0),
            ScratchBlock::ScreenRefresh,
        ])],
        4,
    );
    assert_eq!(m[0].convert_to_number(), 12.0);
}

#[test]
fn custom_blocks_inherit_non_refresh_from_non_refresh_caller() {
    // A normal custom block containing a ScreenRefresh
    let inner = Script::new_custom_block(
        vec![change(X, 1.0), ScratchBlock::ScreenRefresh, change(X, 1.0)],
        0,
        CustomBlockId(1),
        true, // screen refresh
    );

    // A warp custom block calling the normal block
    let outer = Script::new_custom_block(
        vec![
            ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(1), vec![]),
            change(X, 10.0),
        ],
        0,
        CustomBlockId(0),
        false, // not screen refresh
    );

    let m = suite(
        vec![
            inner,
            outer,
            Script::new_green_flag(vec![
                set(X, 0.0),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![]),
            ]),
        ],
        1,
    );
    assert_eq!(m[0].convert_to_number(), 12.0);
}
