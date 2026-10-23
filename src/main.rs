//! A Brainfuck parser and interpreter.
//!
//! Usage: bf-rs <program.bf>
//! If the program reads input (uses ,), the input bytes are read from stdin.

use std::fmt;
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::process;

#[derive(Debug, PartialEq)]
pub enum BFProg {
    IncData,
    DecData,
    IncCurrByte,
    DecCurrByte,
    PrintCurrByte,
    InpByte,
    Sequence(Vec<BFProg>),
    Loop(Box<BFProg>),
}

impl Display for BFProg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BFProg::IncCurrByte => write!(f, "+")?,
            BFProg::DecCurrByte => write!(f, "-")?,
            BFProg::DecData => write!(f, "<")?,
            BFProg::IncData => write!(f, ">")?,
            BFProg::InpByte => write!(f, ",")?,
            BFProg::PrintCurrByte => write!(f, ".")?,
            BFProg::Sequence(body) => {
                for prog in body {
                    write!(f, "{}", prog)?;
                }
            }
            BFProg::Loop(inner) => {
                write!(f, "[")?;
                write!(f, "{}", inner)?;
                write!(f, "]")?;
            }
        }
        Ok(())
    }
}

pub struct BFInterpreter {
    tape: Vec<u8>,
    ptr: usize,
    input: Vec<u8>,
    input_pos: usize,
    output: Vec<u8>,
}

impl BFInterpreter {
    fn new() -> Self {
        Self {
            tape: vec![0],
            ptr: 0,
            input: vec![],
            input_pos: 0,
            output: vec![],
        }
    }

    fn interpret(&mut self, prog: &BFProg) {
        match prog {
            BFProg::IncData => {
                self.ptr += 1;
                if self.tape.len() == self.ptr {
                    self.tape.push(0)
                }
            }
            BFProg::DecData => {
                if self.ptr == 0 {
                    panic!(
                        "Exited because pointer tried to access an index before the first element. Not supported"
                    )
                }
                self.ptr -= 1;
            }
            BFProg::IncCurrByte => {
                self.tape[self.ptr] = self.tape[self.ptr].wrapping_add(1);
            }
            BFProg::DecCurrByte => {
                self.tape[self.ptr] = self.tape[self.ptr].wrapping_sub(1);
            }
            BFProg::PrintCurrByte => {
                self.output.push(self.tape[self.ptr]);
            }
            BFProg::InpByte => {
                // if no input is left, set current box to 0
                if self.input_pos >= self.input.len() {
                    self.tape[self.ptr] = 0;
                } else {
                    self.tape[self.ptr] = self.input[self.input_pos];
                    self.input_pos += 1;
                }
            }
            BFProg::Sequence(items) => {
                for item in items {
                    self.interpret(item);
                }
            }
            BFProg::Loop(inner) => {
                while self.tape[self.ptr] != 0 {
                    self.interpret(inner);
                }
            }
        }
    }
}

pub fn parse_bf(bf_str: &str) -> Option<BFProg> {
    let mut stack: Vec<Vec<BFProg>> = vec![vec![]];

    for c in bf_str.chars() {
        let top = stack.len() - 1;
        match c {
            '>' => stack[top].push(BFProg::IncData),
            '<' => stack[top].push(BFProg::DecData),
            '+' => stack[top].push(BFProg::IncCurrByte),
            '-' => stack[top].push(BFProg::DecCurrByte),
            '.' => stack[top].push(BFProg::PrintCurrByte),
            ',' => stack[top].push(BFProg::InpByte),
            '[' => stack.push(vec![]),
            ']' => {
                if stack.len() == 1 {
                    return None;
                }
                let body = stack.pop().unwrap();
                let curr_top = stack.len() - 1;
                stack[curr_top].push(BFProg::Loop(Box::new(BFProg::Sequence(body))))
            }
            _ => {}
        }
    }

    if stack.len() != 1 {
        return None;
    }
    Some(BFProg::Sequence(stack.pop().unwrap()))
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: bf-rs <program.bf>");
        process::exit(1);
    }
    let source = match std::fs::read_to_string(&args[1]) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("could not read {}: {}", args[1], err);
            process::exit(1);
        }
    };
    let Some(prog) = parse_bf(&source) else {
        eprintln!("{}: brackets don't match", args[1]);
        process::exit(1);
    };

    let mut interpreter = BFInterpreter::new();
    // Only wait for stdin when the program actually reads input.
    if source.contains(',') {
        io::stdin()
            .read_to_end(&mut interpreter.input)
            .expect("failed to read stdin");
    }
    interpreter.interpret(&prog);
    io::stdout()
        .write_all(&interpreter.output)
        .expect("failed to write output");
}

#[test]
fn test_display_1() {
    // [><+-[.]][+]
    let prog = BFProg::Sequence(vec![
        BFProg::Loop(Box::new(BFProg::Sequence(vec![
            BFProg::IncData,
            BFProg::DecData,
            BFProg::IncCurrByte,
            BFProg::DecCurrByte,
            BFProg::Loop(Box::new(BFProg::PrintCurrByte)),
        ]))),
        BFProg::Loop(Box::new(BFProg::IncCurrByte)),
    ]);
    assert_eq!(prog.to_string(), "[><+-[.]][+]".to_string());
}

#[test]
fn test_display_2() {
    // ]><+[] can't be built as the bracket is not well formed. So prog is built without the leading "]"
    let prog = BFProg::Sequence(vec![
        BFProg::IncData,
        BFProg::DecData,
        BFProg::IncCurrByte,
        BFProg::Loop(Box::new(BFProg::Sequence(vec![]))),
    ]); // builds ><+[]
    assert!(prog.to_string() != "]><+[]");
}

#[test]
fn test_bf_interpreter_1() {
    // ,>,[<+>-]<.  adds two input numbers.
    // reads the input into box 1 and 2 respectively
    // Increments box 1 by one unit and decrements box 2 by on unit until box 2 is empty, therby adding 2 numbers
    let add_prog = BFProg::Sequence(vec![
        BFProg::InpByte,
        BFProg::IncData,
        BFProg::InpByte,
        BFProg::Loop(Box::new(BFProg::Sequence(vec![
            BFProg::DecData,
            BFProg::IncCurrByte,
            BFProg::IncData,
            BFProg::DecCurrByte,
        ]))),
        BFProg::DecData,
        BFProg::PrintCurrByte,
    ]);
    let mut interpreter = BFInterpreter::new();
    interpreter.input = vec![5, 6];
    interpreter.interpret(&add_prog);
    assert_eq!(interpreter.output, vec![11]);
}

#[test]
fn test_bf_interpreter_2() {
    // any program you choose. Pick one whose output you can predict by hand.
    // ,.,.-. - at the end wraps 0 to 255
    let prog = BFProg::Sequence(vec![
        BFProg::InpByte,
        BFProg::PrintCurrByte,
        BFProg::InpByte,
        BFProg::PrintCurrByte,
        BFProg::DecCurrByte,
        BFProg::PrintCurrByte,
    ]);
    let mut interpreter = BFInterpreter::new();
    interpreter.input = vec![7];
    interpreter.interpret(&prog);
    assert_eq!(interpreter.output, vec![7, 0, 255]);
}

#[test]
fn test_parser_1() {
    let prog = parse_bf("[><+-[.]][+]").unwrap();
    assert_eq!(prog.to_string(), "[><+-[.]][+]");
}

#[test]
fn test_parser_2() {
    assert!(parse_bf("]><+[]").is_none());
    assert!(parse_bf("[[-]").is_none());
    assert!(parse_bf("[-]]").is_none());
}

#[test]
fn test_parser_3() {
    // Display then parse does not give back the same program.
    // the parser always wraps the top level (and every loop body) in a Sequence.
    let original = BFProg::IncCurrByte;
    let round_trip = parse_bf(&original.to_string()).unwrap();
    assert_ne!(round_trip, original);
    assert_eq!(round_trip, BFProg::Sequence(vec![BFProg::IncCurrByte]));
    assert_eq!(round_trip.to_string(), original.to_string());
}

#[test]
fn test_parser_4() {
    let prog = parse_bf(",>,[<+>-]<.").unwrap();
    let mut interpreter = BFInterpreter::new();
    interpreter.input = vec![5, 6];
    interpreter.interpret(&prog);
    assert_eq!(interpreter.output, vec![11]);
}
