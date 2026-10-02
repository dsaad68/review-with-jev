use std::fmt;

#[derive(Debug, Clone, Copy)]
enum Op {
    Push(i64),
    Add, Sub, Mul,
    Dup, Swap, Over,
    Load(usize), Store(usize),
    JumpIfZero(usize), Jump(usize),
    Print, Halt,
}

#[derive(Debug, PartialEq)]
enum VmError {
    StackUnderflow { pc: usize, needed: usize, depth: usize },
    BadJump(usize),
    BadSlot(usize),
    StepLimit,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackUnderflow { pc, needed, depth } => {
                write!(f, "pc {pc}: needs {needed} values, stack has {depth}")
            }
            VmError::BadJump(t) => write!(f, "jump to {t} is outside the program"),
            VmError::BadSlot(s) => write!(f, "slot {s} does not exist"),
            VmError::StepLimit => write!(f, "step budget exhausted"),
        }
    }
}

enum Flow {
    Next,
    Jump(usize),
    Halt,
}

trait Machine {
    fn step(&mut self, op: Op) -> Result<Flow, VmError>;
}

struct Vm<'p> {
    program: &'p [Op],
    stack: Vec<i64>,
    slots: [i64; 4],
    pc: usize,
    output: Vec<i64>,
}

impl<'p> Vm<'p> {
    fn new(program: &'p [Op]) -> Self {
        Vm {
            program,
            stack: Vec::new(),
            slots: [0; 4],
            pc: 0,
            output: Vec::new(),
        }
    }

    fn require(&self, needed: usize) -> Result<usize, VmError> {
        let depth = self.stack.len();
        if depth < needed {
            return Err(VmError::StackUnderflow { pc: self.pc, needed, depth });
        }
        Ok(depth)
    }

    fn binary(&mut self, f: impl Fn(i64, i64) -> i64) -> Result<(), VmError> {
        let depth = self.require(2)?;
        let rhs = self.stack[depth - 1];
        let lhs = self.stack[depth - 2];
        self.stack.truncate(depth - 2);
        self.stack.push(f(lhs, rhs));
        Ok(())
    }

    fn slot(&mut self, index: usize) -> Result<&mut i64, VmError> {
        self.slots.get_mut(index).ok_or(VmError::BadSlot(index))
    }

    fn run(&mut self, budget: u32) -> Result<&[i64], VmError> {
        let mut remaining = budget;
        while let Some(&op) = self.program.get(self.pc) {
            remaining = remaining.checked_sub(1).ok_or(VmError::StepLimit)?;
            match self.step(op)? {
                Flow::Next => self.pc += 1,
                Flow::Jump(target) if target <= self.program.len() => self.pc = target,
                Flow::Jump(target) => return Err(VmError::BadJump(target)),
                Flow::Halt => break,
            }
        }
        Ok(&self.output)
    }
}

impl Machine for Vm<'_> {
    fn step(&mut self, op: Op) -> Result<Flow, VmError> {
        match op {
            Op::Push(v) => self.stack.push(v),
            Op::Add => self.binary(|a, b| a.wrapping_add(b))?,
            Op::Sub => self.binary(|a, b| a.wrapping_sub(b))?,
            Op::Mul => self.binary(|a, b| a.wrapping_mul(b))?,
            Op::Dup => {
                let depth = self.require(1)?;
                self.stack.push(self.stack[depth - 1]);
            }
            Op::Swap => {
                let depth = self.require(2)?;
                self.stack.swap(depth - 1, depth - 2);
            }
            Op::Over => {
                let depth = self.require(2)?;
                self.stack.push(self.stack[depth - 2]);
            }
            Op::Load(i) => {
                let v = *self.slot(i)?;
                self.stack.push(v);
            }
            Op::Store(i) => {
                self.require(1)?;
                let v = self.stack.pop().unwrap_or_default();
                *self.slot(i)? = v;
            }
            Op::JumpIfZero(target) => {
                self.require(1)?;
                if self.stack.pop() == Some(0) {
                    return Ok(Flow::Jump(target));
                }
            }
            Op::Jump(target) => return Ok(Flow::Jump(target)),
            Op::Print => {
                let depth = self.require(1)?;
                self.output.push(self.stack[depth - 1]);
            }
            Op::Halt => return Ok(Flow::Halt),
        }
        Ok(Flow::Next)
    }
}

fn main() {
    use Op::*;
    let factorial = [
        Push(6), Store(0), Push(1), Store(1),
        Load(0), JumpIfZero(16),
        Load(1), Load(0), Mul, Store(1),
        Load(0), Push(1), Sub, Store(0),
        Jump(4), Halt,
        Load(1), Print, Halt,
    ];
    let programs: [(&str, &[Op]); 4] = [
        ("factorial", &factorial),
        ("underflow", &[Push(1), Add, Print]),
        ("swap", &[Push(3), Push(9), Swap, Over, Sub, Dup, Mul, Print, Halt]),
        ("spin", &[Jump(0)]),
    ];
    for (name, program) in programs {
        let mut vm = Vm::new(program);
        match vm.run(500) {
            Ok(out) => println!("{name}: {out:?}"),
            Err(e) => println!("{name}: error: {e}"),
        }
    }
}
