use std::time::Instant;

use rash_vm::{bytecode::bytecode_instructions::Instruction, data_types::DataValue};

fn main() {
    let mut thread = rash_vm::vm_thread::Thread::new(
        vec![
            Instruction::MemSetToValue {
                ptr: 0,
                value: DataValue::Number(0.0),
            },
            Instruction::MemSetToValue {
                ptr: 1,
                value: DataValue::Number(1.0),
            },
            Instruction::MemSetToValue {
                ptr: 2,
                value: DataValue::Number(0.0),
            },
            Instruction::MemSetToValue {
                ptr: 3,
                value: DataValue::Number(2.0),
            },
            Instruction::MathMod {
                a: 2,
                b: 3,
                result: 4,
            },
            Instruction::MathMultiply {
                a: 3,
                b: 4,
                result: 4,
            },
            Instruction::MemSetToValue {
                ptr: 3,
                value: DataValue::Number(1.0),
            },
            Instruction::MathSubtract {
                a: 4,
                b: 3,
                result: 4,
            },
            Instruction::MemSetToValue {
                ptr: 3,
                value: DataValue::Number(4.0),
            },
            Instruction::MathMultiply {
                a: 4,
                b: 3,
                result: 5,
            },
            Instruction::MathDivide {
                a: 5,
                b: 1,
                result: 3,
            },
            Instruction::MathAdd {
                a: 0,
                b: 3,
                result: 0,
            },
            Instruction::MemSetToValue {
                ptr: 3,
                value: DataValue::Number(2.0),
            },
            Instruction::MathAdd {
                a: 1,
                b: 3,
                result: 1,
            },
            Instruction::MemSetToValue {
                ptr: 3,
                value: DataValue::Number(1.0),
            },
            Instruction::MathAdd {
                a: 2,
                b: 3,
                result: 2,
            },
            Instruction::MemSetToValue {
                ptr: 3,
                value: DataValue::Number(1000000.0),
            },
            Instruction::CompLesser {
                a: 2,
                b: 3,
                result: 3,
            },
            Instruction::JumpToRawLocationIfTrue {
                location: 3,
                condition: 3,
            },
            Instruction::ThreadKill,
        ]
        .into_boxed_slice(),
    );

    let memory: Vec<DataValue> = (0..10).map(|n| DataValue::Number(n as f64)).collect();
    let mut memory = memory.into_boxed_slice();

    let instant = Instant::now();
    thread.run(&mut memory);
    println!("Time passed: {}", instant.elapsed().as_secs_f64());

    memory.iter().for_each(|n| println!("{:?}", n));
}
