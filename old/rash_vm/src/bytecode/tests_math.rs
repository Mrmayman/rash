#[cfg(test)]
mod math_tests {
    use super::bytecode_instructions::*;

    #[test]
    fn add() {
        let mut instruction_counter: usize = 0;

        let mut memory = Box::new([
            DataValue::Number(1.0),
            DataValue::Number(2.0),
            DataValue::Number(-1.0),
        ]);

        Instruction::MathAdd {
            a: DataPointer(0),
            b: DataPointer(1),
            result: DataPointer(2),
        }
        .run(&mut *memory, &mut instruction_counter);

        assert_eq!(memory[2], DataValue::Number(3.0));
    }

    #[test]
    fn subtract() {
        let mut instruction_counter: usize = 0;

        let mut memory = Box::new([
            DataValue::Number(4.0),
            DataValue::Number(1.0),
            DataValue::Number(-1.0),
        ]);

        Instruction::MathSubtract {
            a: DataPointer(0),
            b: DataPointer(1),
            result: DataPointer(2),
        }
        .run(&mut *memory, &mut instruction_counter);

        assert_eq!(memory[2], DataValue::Number(3.0));
    }

    #[test]
    fn multiply() {
        let mut instruction_counter: usize = 0;

        let mut memory = Box::new([
            DataValue::Number(1.5),
            DataValue::Number(2.0),
            DataValue::Number(-1.0),
        ]);

        Instruction::MathMultiply {
            a: DataPointer(0),
            b: DataPointer(1),
            result: DataPointer(2),
        }
        .run(&mut *memory, &mut instruction_counter);

        assert_eq!(memory[2], DataValue::Number(3.0));
    }

    #[test]
    fn divide() {
        let mut instruction_counter: usize = 0;

        let mut memory = Box::new([
            DataValue::Number(6.0),
            DataValue::Number(2.0),
            DataValue::Number(-1.0),
        ]);

        Instruction::MathDivide {
            a: DataPointer(0),
            b: DataPointer(1),
            result: DataPointer(2),
        }
        .run(&mut *memory, &mut instruction_counter);

        assert_eq!(memory[2], DataValue::Number(3.0));
    }

    #[test]
    fn modulo_positive() {
        let mut instruction_counter: usize = 0;

        let mut memory = Box::new([
            DataValue::Number(5.0),
            DataValue::Number(4.0),
            DataValue::Number(-1.0),
        ]);

        Instruction::MathMod {
            a: DataPointer(0),
            b: DataPointer(1),
            result: DataPointer(2),
        }
        .run(&mut *memory, &mut instruction_counter);

        assert_eq!(memory[2], DataValue::Number(1.0));
    }

    #[test]
    fn modulo_negative() {
        let mut instruction_counter: usize = 0;

        let mut memory = Box::new([
            DataValue::Number(5.0),
            DataValue::Number(4.0),
            DataValue::Number(-1.0),
        ]);

        Instruction::MathMod {
            a: DataPointer(0),
            b: DataPointer(1),
            result: DataPointer(2),
        }
        .run(&mut *memory, &mut instruction_counter);

        assert_eq!(memory[2], DataValue::Number(1.0));
    }
}
