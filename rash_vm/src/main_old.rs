fn main() {
    if value == "demo" {
        run_demo();
        drop_strings();
        print_memory();
        std::process::exit(0);
    }

    let loader = match ProjectLoader::new(&std::path::PathBuf::from(value)) {
        Ok(n) => n,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    let scheduler = match loader.build() {
        Ok(n) => n,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    // pollster::block_on(run(scheduler));

    // TODO: Skip screen refresh in some very specific loops.

    drop_strings();
    print_memory();
}

fn run_demo() {
    let mut builder = ProjectBuilder::new();
    compiler::print_func_addresses();

    let memory = MEMORY.lock().unwrap();

    let mut sprite1 = SpriteBuilder::new(SpriteId(0));
    sprite1.add_script(
        &Script::new_custom_block(
            vec![ScratchBlock::ControlRepeat(
                3.0.into(),
                vec![ScratchBlock::VarChange(
                    Ptr(3),
                    ScratchBlock::FunctionGetArg(0).into(),
                )],
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
                    vec![ScratchBlock::FunctionCallScreenRefresh(
                        CustomBlockId(1),
                        vec![1.0.into()],
                    )],
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
            // ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(1), vec![1.0.into()]),
            ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), Vec::new()),
        ]),
        &memory,
    );
    // sprite1.add_script(Script::new_green_flag(block_test::repeat_until()));
    // TODO: Skip screen refresh in some very specific loops.
    // sprite1.add_script(Script::new_green_flag(
    //     block_test::screen_refresh_nested_repeat(),
    // ));
    builder.finish_sprite(sprite1);

    let mut scheduler = builder.finish();

    let mut num_ticks = 1;
    let mut graphics = RunState::default();
    while !scheduler.update(&mut graphics) {
        num_ticks += 1;
        // std::thread::sleep(std::time::Duration::from_millis(1000 / 30))
    }
    println!("Ticks: {num_ticks}");
}

fn print_memory() {
    let lock = MEMORY.lock().unwrap();

    println!("MEMORY: {:X}", lock.as_ptr() as usize);

    // Only print the changed values that aren't zero.
    let print_until_idx = lock
        .iter()
        .enumerate()
        .rev()
        .find(|(_, n)| !matches!(**n, ScratchObject::Number(0.0)))
        .map(|(i, _)| i);
    if let Some(print_until_idx) = print_until_idx {
        for (i, obj) in lock.iter().enumerate().take(print_until_idx + 1) {
            println!("{i}: {obj:?}");
        }
    }
    println!("...: {:?}", ScratchObject::Number(0.0));
}

fn drop_strings() {
    let mut strings_buf = STRINGS_TO_DROP.lock().unwrap();
    let mut strings: Vec<[i64; 3]> = Vec::new();
    std::mem::swap(strings_buf.as_mut(), &mut strings);

    for string in strings {
        let _string: String = unsafe { std::mem::transmute(string) };
        // println!("Dropping string {_string}");
        // Drop string
    }
}
