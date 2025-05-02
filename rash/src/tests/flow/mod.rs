#[cfg(test)]
mod tests {
    use rash_render::{Run, RunState, SpriteId};

    use crate::{
        compiler::{ScratchBlock, MEMORY},
        input_primitives::Ptr,
        scheduler::{CustomBlockId, ProjectBuilder, Script, SpriteBuilder},
    };

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
                    ScratchBlock::VarSet(Ptr(3), 0.5.into()),
                    ScratchBlock::ControlRepeat(
                        5.0.into(),
                        vec![
                            ScratchBlock::FunctionCallScreenRefresh(
                                CustomBlockId(1),
                                vec![2.0.into()],
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
                ScratchBlock::VarSet(Ptr(3), 0.5.into()),
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

        assert_eq!(num_ticks, 21);
        assert_eq!(memory[3].convert_to_number(), 30.5);
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

        assert_eq!(num_ticks, 16);
        assert_eq!(memory[3].convert_to_number(), 24.5);
    }
}
