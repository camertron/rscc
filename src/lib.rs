use std::str::FromStr;

pub enum Instruction {
    LDA(LDA),
    LDC(LDC),
    STA(STA),
    INP(INP),
    OUT(OUT),
    ADC(ADC),
    ADD(ADD),
    SUB(SUB),
    MUL(MUL),
    DIV(DIV),
    BRU(BRU),
    BPA(BPA),
    BNA(BNA),
    BZA(BZA),
    STP(STP),
}

impl Instruction {
    pub fn lineno(self: &Self) -> usize {
        match self {
            Instruction::LDA(lda) => lda.lineno,
            Instruction::LDC(ldc) => ldc.lineno,
            Instruction::STA(sta) => sta.lineno,
            Instruction::INP(inp) => inp.lineno,
            Instruction::OUT(out) => out.lineno,
            Instruction::ADC(adc) => adc.lineno,
            Instruction::ADD(add) => add.lineno,
            Instruction::SUB(sub) => sub.lineno,
            Instruction::MUL(mul) => mul.lineno,
            Instruction::DIV(div) => div.lineno,
            Instruction::BRU(bru) => bru.lineno,
            Instruction::BPA(bpa) => bpa.lineno,
            Instruction::BNA(bna) => bna.lineno,
            Instruction::BZA(bza) => bza.lineno,
            Instruction::STP(stp) => stp.lineno,
        }
    }
}

// Load value from location into accumulator.
//
// Example: LDA 5 loads the value stored in memory location 5 into the accumulator.
#[derive(Debug)]
pub struct LDA {
    pub location: u32,
    lineno: usize,
}

// Load constant value into accumulator.
//
// Example: LDC 5 loads the literal number 5 into the accumulator.
#[derive(Debug)]
pub struct LDC {
    pub value: f64,
    lineno: usize,
}

// Store accumulator into location.
//
// Example: STA 5 writes the current value inside the accumulator into memory location 5.
#[derive(Debug)]
pub struct STA {
    pub location: u32,
    lineno: usize,
}

// Input value from keyboard and store at location.
//
// Example: INP 5 prompts the user for input via the keyboard. The inputted number is then
// stored in memory location 5.
#[derive(Debug)]
pub struct INP {
    pub location: u32,
    lineno: usize,
}

// Output value from location onto the screen.
//
// Example: OUT 5 causes the value stored at memory location 5 to show up on the terminal screen.
#[derive(Debug)]
pub struct OUT {
    pub location: u32,
    lineno: usize,
}

// Add constant value to accumulator.
//
// Example: ADC 5 adds the literal number 5 to the existing accumulator value. If the accumulator
// originally contains 10, after ADC 5 it will contain a value of 15.
#[derive(Debug)]
pub struct ADC {
    pub value: f64,
    lineno: usize,
}

// Add value stored at location into accumulator.
//
// Example: ADD 5 adds the value stored at memory location 5 to the existing accumulator value. If
// the accumulator originally contains 10 and memory location 5 contains a value of 8, after ADD 5
// the accumulator will contain a value of 18.
#[derive(Debug)]
pub struct ADD {
    pub location: u32,
    lineno: usize,
}

// Subtract value stored in location from accumulator.
//
// Example: SUB 5 subtracts the value stored at memory location 5 from the existing accumulator value.
// If the accumulator originally contains 10 and memory location 5 contains a value of 8, after SUB 5
// the accumulator will contain a value of 2.
#[derive(Debug)]
pub struct SUB {
    pub location: u32,
    lineno: usize,
}

// Multiply accumulator by value stored in location.
//
// Example: MUL 5 multiplies the existing accumulator value by the value stored at memory location 5. If
// the accumulator originally contains 10 and memory location 5 contains a value of 8, after MUL 5 the
// accumulator will contain a value of 80.
#[derive(Debug)]
pub struct MUL {
    pub location: u32,
    lineno: usize,
}

// Divide accumulator by value stored in location.
//
// Example: DIV 5 divides the existing accumulator value by the value stored at memory location 5. If the
// accumulator originally contains 20 and memory location 5 contains a value of 4, after DIV 5 the
// accumulator will contain a value of 5.
#[derive(Debug)]
pub struct DIV {
    pub location: u32,
    lineno: usize,
}

// Branch to location.
//
// Example: BRU 5 causes execution to jump to instruction 5, skipping all the instructions between 5 and the
// current instruction. Note that it is perfectly acceptable to jump backwards as well as forwards, i.e. to
// an instruction before or after the current one.
#[derive(Debug)]
pub struct BRU {
    pub location: u32,
    lineno: usize,
}

// Branch to location if accumulator is positive.
//
// Example: BPA 5 jumps to location 5 if and only if the current accumulator value is positive (i.e. greater
// than but not equal to zero). If the accumulator contains a value of 2, after BPA 5 execution will resume
// from instruction 5. If the accumulator contains a value of 0 or less, after BPA 5 execution will continue
// with the instruction immediately following the current one. Note that it is perfectly acceptable to jump
// backwards as well as forwards, i.e. to an instruction before or after the current one.
#[derive(Debug)]
pub struct BPA {
    pub location: u32,
    lineno: usize,
}

// Branch to location if accumulator is negative.
//
// Example: BNA 5 jumps to location 5 if and only if the current accumulator value is negative (i.e. less than
// but not equal to zero). If the accumulator contains a value of -2, after BNA 5 execution will resume from
// instruction 5. If the accumulator contains a value of 0 or greater, after BNA 5 execution will continue with
// the instruction immediately following the current one. Note that it is perfectly acceptable to jump backwards
// as well as forwards, i.e. to an instruction before or after the current one.
#[derive(Debug)]
pub struct BNA {
    pub location: u32,
    lineno: usize,
}

// Branch to location if accumulator is zero.
//
// Example: BZA 5 jumps to location 5 if and only if the current accumulator value is zero. If the accumulator
// contains a value of 0, after BZA 5 execution will resume from instruction 5. If the accumulator contains any
// other positive or negative value, after BZA 5 execution will continue with the instruction immediately following
// the current one. Note that it is perfectly acceptable to jump backwards as well as forwards, i.e. to an
// instruction before or after the current one.
#[derive(Debug)]
pub struct BZA {
    pub location: u32,
    lineno: usize,
}

// Stop execution.
//
// Example: STP terminates your program. All programs must have STP as the last instruction.
#[derive(Debug)]
pub struct STP {
    lineno: usize,
}

#[derive(Debug)]
pub enum DiagnosticType {
    InvalidOpcode,
    InvalidOperand,
    MissingOperand,
    TooManyOperands,
    MissingStp,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub ty: DiagnosticType,
    pub start: usize,
    pub end: usize,
}

fn last_n<'a>(n: usize, list: &Vec<&'a str>) -> Vec<&'a str> {
    if list.len() > n {
        list[(list.len() - n)..].to_vec()
    } else {
        list.clone()
    }
}

fn first_n<'a>(n: usize, list: &Vec<&'a str>) -> Vec<&'a str> {
    if list.len() > n {
        list[0..n].to_vec()
    } else {
        list.clone()
    }
}

fn annotate_range(source: &str, start: usize, end: usize, message: &str) -> String {
    let bol = match source[0..start].rfind("\n") {
        Some(pos) => (pos + 1).clamp(0, source.len() - 1),
        None => 0
    };

    let eol = match source[start..].find("\n") {
        Some(pos) => (start + pos).clamp(0, source.len() - 1),
        None => source.len() - 1
    };

    let before_lines = &source[0..bol].split("\n").filter(|str| {
        str.trim().len() > 0
    }).collect::<Vec<&str>>();

    let before_lines_keep = last_n(2, before_lines);
    let mut lineno = before_lines.len().saturating_sub(2) + 1;
    let before_lines_keep_linenos: Vec<String> = before_lines_keep.iter().map(|line| {
        let result = format!("{}. {}", lineno, line);
        lineno += 1;
        result
    }).collect();

    let current_lineno = lineno;
    lineno += 1;

    let after_lines = source[(eol + 1)..].split("\n").filter(|str| {
        str.trim().len() > 0
    }).collect::<Vec<&str>>();

    let after_lines_keep = first_n(2, &after_lines);
    let after_lines_keep_linenos: Vec<String> = after_lines_keep.iter().map(|line| {
        let result = format!("{}. {}", lineno, line);
        lineno += 1;
        result
    }).collect();

    let annotated = format!(
        "{before}\n{lineno}. {line}\n{leading_ws}^{dashes} {message}\n{after}",
        before = before_lines_keep_linenos.join("\n"),
        line = &source[bol..eol],
        lineno = current_lineno,
        leading_ws = " ".repeat(start - bol + format!("{}", lineno).len() + 2),
        dashes = "-".repeat((end - start).saturating_sub(1)),
        message = message,
        after = after_lines_keep_linenos.join("\n")
    );

    annotated.trim().to_string()
}

impl Diagnostic {
    pub fn new(ty: DiagnosticType, start: usize, end: usize) -> Self {
        Diagnostic { ty, start, end }
    }

    pub fn annotate(self: &Self, source: &str) -> String {
        match self.ty {
            DiagnosticType::InvalidOpcode => {
                annotate_range(source, self.start, self.end, "Invalid opcode")
            }

            DiagnosticType::MissingOperand => {
                annotate_range(source, self.start, self.end, "Missing operand")
            }

            DiagnosticType::InvalidOperand => {
                annotate_range(source, self.start, self.end, "Invalid operand, expected a number")
            }

            DiagnosticType::TooManyOperands => {
                annotate_range(source, self.start, self.end, "Only one operand expected")
            }

            DiagnosticType::MissingStp => {
                annotate_range(source, self.start, self.end, "Program must contain at least one STP instruction")
            }
        }
    }
}

pub struct ParseResult {
    pub instructions: Vec<Instruction>,
    pub diagnostics: Vec<Diagnostic>,
    pub code: String,
}

impl ParseResult {
    pub fn new(instructions: Vec<Instruction>, diagnostics: Vec<Diagnostic>, code: String) -> Self {
        ParseResult { instructions, diagnostics, code }
    }
}

pub fn parse(str: &str) -> ParseResult {
    let lines = str.split('\n');
    let mut instructions: Vec<Instruction> = vec![];
    let mut diagnostics: Vec<Diagnostic> = vec![];
    let mut cur_pos: usize = 0;
    let mut found_stp = false;

    for (mut lineno, line) in lines.enumerate() {
        lineno += 1;

        // skip comments
        if line.len() == 0 || line.starts_with('#') {
            cur_pos += line.len() + 1;
            continue;
        }

        let parts = line.split_inclusive(' ').collect::<Vec<&str>>();
        let mut opcode: Option<&str> = None;
        let mut operands: Vec<(&str, usize)> = vec![];
        let mut operand_start = cur_pos;

        for part in parts {
            let trimmed_part = part.trim();

            if trimmed_part.len() > 0 {
                if opcode.is_some() {
                    operands.push((trimmed_part, operand_start));
                } else {
                    opcode = Some(trimmed_part);
                    operand_start += part.len();
                }
            } else {
                operand_start += part.len();
            }
        }

        match opcode {
            Some("STP") => {
                if operands.len() > 0 {
                    let first_op_start = operands[0].1;

                    diagnostics.push(
                        Diagnostic::new(
                            DiagnosticType::TooManyOperands,
                            first_op_start,
                            cur_pos + line.len()
                        )
                    )
                }
            }

            Some(_) => {
                if operands.len() > 1 {
                    let first_extra_op_start = operands[1].1;

                    diagnostics.push(
                        Diagnostic::new(
                            DiagnosticType::TooManyOperands,
                            first_extra_op_start,
                            cur_pos + line.len()
                        )
                    )
                }
            },

            None => {}
        }

        let operand = if operands.len() > 0 {
            Some(operands[0])
        } else {
            None
        };

        match opcode {
            Some("LDA") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::LDA(LDA { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("LDC") => match parse_operand::<f64>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::LDC(LDC { lineno, value: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("STA") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::STA(STA { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("OUT") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::OUT(OUT { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("INP") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::INP(INP { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("ADC") => match parse_operand::<f64>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::ADC(ADC { lineno, value: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("ADD") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::ADD(ADD { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("SUB") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::SUB(SUB { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("MUL") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::MUL(MUL { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("DIV") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::DIV(DIV { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("BRU") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::BRU(BRU { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("BPA") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::BPA(BPA { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("BNA") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::BNA(BNA { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("BZA") => match parse_operand::<u32>(operand) {
                Some(Ok(operand)) => instructions.push(
                    Instruction::BZA(BZA { lineno, location: operand })
                ),

                Some(Err(diag)) => diagnostics.push(diag),

                None => diagnostics.push(
                    Diagnostic::new(DiagnosticType::MissingOperand, cur_pos + 3, cur_pos + 3)
                )
            },

            Some("STP") => {
                instructions.push(Instruction::STP(STP { lineno }));
                found_stp = true;
            },

            Some(opcode) => diagnostics.push(
                Diagnostic::new(DiagnosticType::InvalidOpcode, cur_pos, cur_pos + opcode.len())
            ),

            None => diagnostics.push(
                Diagnostic::new(DiagnosticType::InvalidOpcode, cur_pos, cur_pos)
            )
        }

        cur_pos += line.len() + 1;
    }

    if !found_stp {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticType::MissingStp,
                str.len(),
                str.len()
            )
        )
    }

    return ParseResult::new(instructions, diagnostics, str.to_string());
}

fn parse_operand<T: FromStr>(operand: Option<(&str, usize)>) -> Option<Result<T, Diagnostic>> {
    match operand {
        Some((op, start)) => {
            match op.parse::<T>() {
                Ok(op) => Some(Ok(op)),
                Err(_) => Some(
                    Err(
                        Diagnostic::new(
                            DiagnosticType::InvalidOperand,
                            start,
                            start + op.len()
                        )
                    )
                )
            }
        },

        _ => None
    }
}
