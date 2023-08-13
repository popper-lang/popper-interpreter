use crate::value::Literal;
use crate::value::StrPtr;

#[derive(Debug, Clone, PartialEq)]
/// different instruction of bytecode
pub enum Instruction {
    /// Name: Push Literal
    /// Opcode: 0x01
    /// Operand:
    /// - Literal: a literal
    PushLiteral(Literal),
    /// Name: Push Variable
    /// Opcode: 0x02
    /// Operand:
    /// - StrPtr: a pointer of a str
    PushVariable(StrPtr),
    /// Name: Jump If False
    /// Opcode: 0x03
    /// Operand:
    /// - bool: if it is included on the asm code
    /// - Vec<Instruction>: the byte code to execute if the condition is false
    JIF(bool, Vec<Instruction>),
    /// Name: Jump
    /// Opcode: 0x04
    /// Operand:
    /// - bool: if it is included on the asm code
    /// - Vec<Instruction>: the byte code to execute
    Jmp(bool, Vec<Instruction>),
    /// Name: Jump If True
    /// Opcode: 0x05
    /// Operand:
    /// - bool: if it is included on the asm code
    /// - Vec<Instruction>: the byte code to execute if the condition is true
    JIT(bool, Vec<Instruction>),
    /// Name: Call function
    /// Opcode: 0x0F
    /// Operand:
    /// - StrPtr: a pointer of a str, the name of the function
    Call(StrPtr),
    /// Name: Store variable
    /// Opcode: 0x06
    /// Operand:
    /// - StrPtr: a pointer of a str, the name of the variable
    Store(StrPtr),
    /// Name: Add
    /// Opcode: 0x07
    Add,
    /// Name: Subtract
    /// Opcode: 0x08
    Sub,
    /// Name: Multiply
    /// Opcode: 0x09
    Mul,
    /// Name: Divide
    /// Opcode: 0x0A
    Div,
    /// Name: Negation
    /// Opcode: 0x0B
    Neg,
    /// Name: Return
    /// Opcode: 0x0C
    Return,
    /// Name: Nop (do nothing)
    /// Opcode: 0x0D
    Nop,
    /// Name: Pop
    /// Opcode: 0x0E
    Pop,
    /// Name: End Jump
    /// Opcode: 0x10
    EndJmp
}

/// bytecode trait for compile rust data to bytecode data
pub trait Bytecode: Sized {
    fn to_bytecode(&self) -> Vec<u8>;
    fn from_bytecode(bytecode: Vec<u8>) -> Self;
    fn write_file(&self, file: &str) {
        let bytecode = self.to_bytecode();
        std::fs::write(file, bytecode).expect("Unable to write file");
    }

    fn read_file(file: &str) -> Self {
        let bytecode = std::fs::read(file).expect("Unable to read file");
        Self::from_bytecode(bytecode)
    }
}


/// jump format
type JFormat = (bool, Vec<Instruction>);

impl Bytecode for Instruction {
    fn to_bytecode(&self) -> Vec<u8> {
        match self {
            Instruction::PushLiteral(lit) => {
                let mut bytecode = vec![0x01];
                bytecode.extend(lit.to_bytecode());
                bytecode
            },
            Instruction::PushVariable(name) => {
                let mut bytecode = vec![0x02];
                bytecode.extend(name.to_bytecode());
                bytecode
            },
            Instruction::JIF(jump, instrs) => {
                let mut bytecode = vec![0x03];
                let t = (jump.clone(), instrs.clone());
                bytecode.extend(t.to_bytecode());
                bytecode
            }
            Instruction::Jmp(jump, instrs) => {
                let mut bytecode = vec![0x04];
                let t = (jump.clone(), instrs.clone());
                bytecode.extend(t.to_bytecode());
                bytecode
            }
            Instruction::Call(name) => {
                let mut bytecode = vec![0x05];
                bytecode.extend(name.to_bytecode());
                bytecode
            },
            Instruction::Store(name) => {
                let mut bytecode = vec![0x06];
                bytecode.extend(name.to_bytecode());
                bytecode
            },

            Instruction::Add => vec![0x07],
            Instruction::Sub => vec![0x08],
            Instruction::Mul => vec![0x09],
            Instruction::Div => vec![0x0A],
            Instruction::Neg => vec![0x0B],
            Instruction::Return => vec![0x0C],
            Instruction::Nop => vec![0x0D],
            Instruction::Pop => vec![0x0E],
            Instruction::JIT(jump, instrs) => {
                let mut bytecode = vec![0x0F];
                let t = (jump.clone(), instrs.clone());
                bytecode.extend(t.to_bytecode());
                bytecode
            }
            Instruction::EndJmp => vec![0x10]
        }
    }

    fn from_bytecode(bytecode: Vec<u8>) -> Self {
        match bytecode[0] {
            0x01 => Instruction::PushLiteral(Literal::from_bytecode(bytecode[1..].to_vec())),
            0x02 => Instruction::PushVariable(StrPtr::from_bytecode(bytecode[1..].to_vec())),
            0x03 => {
                let (is_included, instrs) = JFormat::from_bytecode(bytecode[1..].to_vec());
                Instruction::JIF(is_included, instrs)
            },
            0x04 => {
                let (is_included, instrs) = JFormat::from_bytecode(bytecode[1..].to_vec());
                Instruction::Jmp(is_included, instrs)
            },
            0x05 => Instruction::Call(StrPtr::from_bytecode(bytecode[1..].to_vec())),
            0x06 => Instruction::Store(StrPtr::from_bytecode(bytecode[1..].to_vec())),
            0x07 => Instruction::Add,
            0x08 => Instruction::Sub,
            0x09 => Instruction::Mul,
            0x0A => Instruction::Div,
            0x0B => Instruction::Neg,
            0x0C => Instruction::Return,
            0x0D => Instruction::Nop,
            0x0E => Instruction::Pop,
            0x0F => {
                let (is_included, instrs) = JFormat::from_bytecode(bytecode[1..].to_vec());
                Instruction::JIT(is_included, instrs)
            },
            0x10 => Instruction::EndJmp,
            e => panic!("Invalid bytecode: {}", e)
        }
    }
}



