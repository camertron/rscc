pub enum Instruction {
    LDC(LDC),
    STA(STA),
    OUT(OUT),
    STP(STP),
}

#[derive(Debug)]
pub struct LDC {
    pub operand: i32,
}

#[derive(Debug)]
pub struct STA {
    pub operand: i32,
}

#[derive(Debug)]
pub struct OUT {
    pub operand: i32,
}

#[derive(Debug)]
pub struct STP;

pub struct InvalidInstruction {
    pub instruction: String,
}

pub enum ParseError {
    InvalidInstruction(InvalidInstruction),
}

pub type ParseResult = Result<Vec<Instruction>, ParseError>;

pub fn parse(str: &str) -> ParseResult {
    let lines = str.split('\n');
    let mut instructions: Vec<Instruction> = vec![];

    for line in lines {
        if line.len() == 0 || line.starts_with('#') {
            continue;
        }

        let parts = line.split(' ').collect::<Vec<&str>>();

        match parts[0] {
            "LDC" => instructions.push(Instruction::LDC(LDC { operand: parts[1].parse::<i32>().unwrap() })),
            "STA" => instructions.push(Instruction::STA(STA { operand: parts[1].parse::<i32>().unwrap() })),
            "OUT" => instructions.push(Instruction::OUT(OUT { operand: parts[1].parse::<i32>().unwrap() })),
            "STP" => instructions.push(Instruction::STP(STP { })),
            _ => return Err(
                ParseError::InvalidInstruction(
                    InvalidInstruction { instruction: parts[0].to_string() }
                )
            )
        }
    }

    return Ok(instructions);
}
