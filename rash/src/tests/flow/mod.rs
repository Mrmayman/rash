// Note: Scratch loops omit a screen refresh
// if there aren't any graphics/time related
// operations inside.

// But here, we need to test screen refresh.
// However we can't use graphical operations
// inside a test environment, so we just use
// an explicit ScreenRefresh

#[cfg(test)]
mod tests {
    use rash_render::{Run, RunState, SpriteId};

    use crate::{
        compiler::{ScratchBlock, MEMORY},
        input_primitives::Ptr,
        scheduler::{CustomBlockId, ProjectBuilder, Script, SpriteBuilder},
    };

    use super::*;

    #[test]
    fn custom_block_screen_refresh() {
        let memory = MEMORY.lock().unwrap();

        let mut builder = ProjectBuilder::new();

        let mut sprite1 = SpriteBuilder::new(SpriteId(0));
        sprite1.add_script(
            &Script::new_custom_block(
                vec![ScratchBlock::ControlRepeat(
                    3.0.into(),
                    vec![
                        ScratchBlock::VarChange(Ptr(3), ScratchBlock::FunctionGetArg(0).into()),
                        ScratchBlock::ScreenRefresh,
                    ],
                )],
                1,
                CustomBlockId(1),
                true,
            ),
            &memory,
        );
        sprite1.add_script(
            &Script::new_custom_block(
                vec![
                    ScratchBlock::VarSet(Ptr(3), 0.0.into()),
                    ScratchBlock::ControlRepeat(
                        5.0.into(),
                        vec![
                            ScratchBlock::FunctionCallScreenRefresh(
                                CustomBlockId(1),
                                vec![1.0.into()],
                            ),
                            ScratchBlock::ScreenRefresh,
                        ],
                    ),
                ],
                0,
                CustomBlockId(0),
                true,
            ),
            &memory,
        );
        sprite1.add_script(
            &Script::new_green_flag(vec![
                ScratchBlock::VarSet(Ptr(3), 0.0.into()),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), Vec::new()),
            ]),
            &memory,
        );
        builder.finish_sprite(sprite1);
        let mut scheduler = builder.finish();

        let mut num_ticks = 1;
        let mut graphics = RunState::default();
        while !scheduler.update(&mut graphics) {
            num_ticks += 1;
        }

        assert_eq!(by_two(num_ticks), 21);
        assert_eq!(memory[3].convert_to_number(), 15.0);
    }

    #[test]
    fn nested_loop_screen_refresh() {
        let memory = MEMORY.lock().unwrap();

        let mut builder = ProjectBuilder::new();

        let mut sprite1 = SpriteBuilder::new(SpriteId(0));
        sprite1.add_script(
            &Script::new_green_flag(vec![
                ScratchBlock::VarSet(Ptr(3), 0.5.into()),
                ScratchBlock::ControlRepeat(
                    3.0.into(),
                    vec![
                        ScratchBlock::ControlRepeat(
                            4.0.into(),
                            vec![
                                ScratchBlock::VarChange(Ptr(3), 2.0.into()),
                                ScratchBlock::ScreenRefresh,
                            ],
                        ),
                        ScratchBlock::ScreenRefresh,
                    ],
                ),
            ]),
            &memory,
        );
        builder.finish_sprite(sprite1);
        let mut scheduler = builder.finish();

        let mut num_ticks = 1;
        let mut graphics = RunState::default();
        while !scheduler.update(&mut graphics) {
            num_ticks += 1;
        }

        assert_eq!(by_two(num_ticks), 16);
        assert_eq!(memory[3].convert_to_number(), 24.5);
    }
}

#[cfg(test)]
fn by_two(n: i32) -> i32 {
    if n % 2 == 0 {
        // If n is even, standard division works
        n / 2
    } else {
        // If n is odd, integer division truncates, so add 1
        n / 2 + 1
    }
}
