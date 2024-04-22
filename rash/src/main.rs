use std::time::Instant;

use rash_vm::{
    bytecode::instructions::{DataPointer, Instruction, JumpPoint},
    data_types::ScratchObject,
};

fn _run() {
    let mut thread = rash_vm::vm_thread::Thread::new(
        vec![
            Instruction::MemSetToValue {
                ptr: DataPointer(0),
                value: ScratchObject::Number(0.0),
            },
            Instruction::MemSetToValue {
                ptr: DataPointer(1),
                value: ScratchObject::Number(1.0),
            },
            Instruction::MemSetToValue {
                ptr: DataPointer(3),
                value: ScratchObject::Number(0.0),
            },
            Instruction::JumpDefinePoint {
                place: JumpPoint(0),
            },
            Instruction::MathMod {
                a: ScratchObject::Pointer(3),
                b: ScratchObject::Number(2.0),
                result: DataPointer(2),
            },
            Instruction::MathMultiply {
                a: ScratchObject::Number(2.0),
                b: ScratchObject::Pointer(2),
                result: DataPointer(2),
            },
            Instruction::MathSubtract {
                a: ScratchObject::Pointer(2),
                b: ScratchObject::Number(1.0),
                result: DataPointer(2),
            },
            Instruction::MathMultiply {
                a: ScratchObject::Number(4.0),
                b: ScratchObject::Pointer(2),
                result: DataPointer(4),
            },
            Instruction::MathDivide {
                a: ScratchObject::Pointer(4),
                b: ScratchObject::Pointer(1),
                result: DataPointer(4),
            },
            Instruction::MathAdd {
                a: ScratchObject::Pointer(0),
                b: ScratchObject::Pointer(4),
                result: DataPointer(0),
            },
            Instruction::MathAdd {
                a: ScratchObject::Pointer(1),
                b: ScratchObject::Number(2.0),
                result: DataPointer(1),
            },
            Instruction::MathAdd {
                a: ScratchObject::Pointer(3),
                b: ScratchObject::Number(1.0),
                result: DataPointer(3),
            },
            Instruction::CompLesser {
                a: ScratchObject::Pointer(3),
                b: ScratchObject::Number(1_000_000.0),
                result: DataPointer(4),
            },
            Instruction::JumpToPointIfTrue {
                place: JumpPoint(0),
                condition: ScratchObject::Pointer(4),
            },
            Instruction::ThreadKill,
        ]
        .into_boxed_slice(),
    );

    let memory: Vec<ScratchObject> = (0..10).map(|n| ScratchObject::Number(n as f64)).collect();
    let mut memory = memory.into_boxed_slice();

    thread.optimize();
    let instant = Instant::now();
    thread.run(&mut memory);
    println!("Time passed: {}", instant.elapsed().as_secs_f64());

    memory.iter().for_each(|n| println!("{:?}", n));
}

fn main() {
    let mut project = match std::env::args().nth(1) {
        Some(n) => rash_loader_sb3::load::ProjectFile::open(&std::path::PathBuf::from(n)),
        None => {
            eprintln!("Pass an argument to a project to be run");
            return;
        }
    }
    .unwrap();

    project.load();
}
