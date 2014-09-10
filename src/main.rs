/*
 * Interpreter for the Brainfuck programming language specified
 * at http://www.muppetlabs.com/~breadbox/bf/
 *
 * @author: Ethan Lewis <3thanlewis@gmail.com>
 */

use std::os;
use std::io;

static STATE_SIZE : uint = 30000u;

fn main() {
    let mut intr = Interpreter::new();

    let args = os::args();
    if args.len() > 1 {
        intr.eval(read_file(args[1].as_slice()));
    } else {
        intr.repl();
    }
}

fn read_file(filename: &str) -> Vec<char> {
    let path = Path::new(filename);
    let mut file = io::BufferedReader::new(io::File::open(&path));
    let data = file.read_to_end().ok().expect("file cannot be read");
    data.iter().map(|c:&u8| *c as char).collect()
}

struct Interpreter<R: Reader> {
    ptr    : uint,
    state  : [u8, ..STATE_SIZE],
    prompt : &'static str,
    buf    : io::BufferedReader<R>
}

impl Interpreter<io::stdio::StdReader> {

    fn new() -> Interpreter<io::stdio::StdReader> {
        Interpreter {
            ptr    : 0u,
            state  : [0u8, ..STATE_SIZE],
            prompt : "BF-> ",
            buf    : io::stdio::stdin()
        }
    }

    fn repl(&mut self) {
        loop {
            print!("{}", self.prompt);
            let line = match self.buf.read_line() {
                Err(m) => { println!("{}", m); break },
                x      => x.unwrap()
            };
            self.eval(line.as_slice().chars().collect());
            println!("");
        }
    }

    fn eval(&mut self, chars: Vec<char>) {
        let mut tokens = Interpreter::tokenize(chars);
        let mut code_ptr = 0u;

        // Construct syntax tree and replace loop brackets with jumps
        let mut stack: Vec<uint> = Vec::new();
        while code_ptr < tokens.len() {
            match tokens[code_ptr] {
                FWRD => {
                    stack.push(code_ptr);
                }
                BACK => {
                    if stack.len() > 0 {
                        let start = stack.pop().unwrap();
                        *tokens.get_mut(start)    = JMPF(code_ptr + 1);
                        *tokens.get_mut(code_ptr) = JMPT(start);
                    } else {
                        fail!("unexpected close bracket");
                    }
                }
                _ => {}
            }
            code_ptr += 1;
        }

        code_ptr = 0u;
        while code_ptr < tokens.len() {
            assert!(self.is_ptr_valid(), "pointer out of bounds");
            match tokens[code_ptr] {
                NEXT => {
                    self.ptr += 1
                }
                PREV => {
                    self.ptr -= 1
                }
                INCR => {
                    self.state[self.ptr] += 1
                }
                DECR => {
                    self.state[self.ptr] -= 1
                }
                OUTP => {
                    print!("{}", self.state[self.ptr] as char)
                }
                INPT => {
                    let c = self.buf.read_char().ok().expect("invalid character");
                    self.state[self.ptr] = c as u8
                }
                JMPT(dest) => {
                    if self.state[self.ptr] != 0u8 { code_ptr = dest }
                }
                JMPF(dest) => {
                    if self.state[self.ptr] == 0u8 { code_ptr = dest }
                }
                NOOP => {
                    
                }
                _ => fail!("not implemented")
            }
            code_ptr += 1;
        }
    }

    fn tokenize(chars: Vec<char>) -> Vec<Cmd> {
        chars.iter().map(|c: &char| Cmd::from_char(*c)).collect()
    }

    fn is_ptr_valid(&self) -> bool {
        self.ptr < self.state.len()
    }

}

enum Cmd {
    NEXT,
    PREV,
    INCR,
    DECR,
    OUTP,
    INPT,
    FWRD,
    BACK,
    JMPT(uint),
    JMPF(uint),
    NOOP
}

impl Cmd {
    fn from_char(c: char) -> Cmd {
        match c {
            '>' => NEXT, // >
            '<' => PREV, // <
            '+' => INCR, // +
            '-' => DECR, // -
            '.' => OUTP, // .
            ',' => INPT, // ,
            '[' => FWRD, // [
            ']' => BACK, // ]
            _   => NOOP
        }
    }
}
