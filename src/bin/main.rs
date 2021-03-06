use std::io::{self,Read,BufRead,Write};
use rand::{self,distributions};
use std::fmt;
use std::error;
use std::num::ParseIntError;

const WIDTH: usize = 80;
const HEIGHT: usize = 30;

type Playfield = [[i32;WIDTH];HEIGHT];

#[derive(Clone,Copy)]
enum Direction {
    Right, Left, Up, Down
}

impl distributions::Distribution<Direction> for distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        use Direction::*;
        match rng.gen_range::<u8,_>(0..=3) {
            0 => Right,
            1 => Left,
            2 => Up,
            _ => Down
        }
    }
}

struct State {
    playfield: Playfield,
    line: usize,
    col: usize,
    stack: Vec<i32>,
    dir: Direction,
    str_mode: bool
}

#[derive(Debug)]
enum Error { SourceTooTall, SourceTooWide(usize), NoArgs, ReadSourceFailed(io::Error), WriteFailed(io::Error), ParseIntFailed(ParseIntError) }

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            SourceTooTall => write!(f, "program contains too many lines (> 256)"),
            SourceTooWide(i) => write!(f, "program contains too many columns (> 256) on line {}", i+1),
            NoArgs => write!(f, "no source file provided"),
            ReadSourceFailed(_) => write!(f, "failed to read source file"),
            WriteFailed(_) => write!(f, "failed to write to stdout"),
            ParseIntFailed(_) => write!(f, "invalid numeric input")
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            ReadSourceFailed(e) => Some(e),
            WriteFailed(e) => Some(e),
            ParseIntFailed(e) => Some(e),
            _ => None
        }
    }
}

fn parse_program(source: &str) -> Result<Playfield,Error> {
    use Error::*;

    let lines: Vec<_> = source.split('\n').collect();
    if lines.len() > HEIGHT { return Err(SourceTooTall); }

    let mut program = [[0;WIDTH];HEIGHT];

    for (i, line) in lines.iter().enumerate() {
        let chars: Vec<_> = line.chars().map(|c| c as i32).collect();
        let len = chars.len();
        if len > WIDTH { return Err(SourceTooWide(i)); }
        program[i][0..len].copy_from_slice(&chars);
    }

    Ok(program)
}

impl State {
    fn init(playfield: Playfield) -> Self {
        State {
            playfield,
            line: 0,
            col: 0,
            stack: Vec::new(),
            dir: Direction::Right,
            str_mode: false,
        }
    }

    fn pop(&mut self) -> i32 {
        self.stack.pop().unwrap_or(0)
    }

    fn push(&mut self, val: i32) {
        self.stack.push(val);
    }

    fn mov(&mut self) {
        use Direction::*;
        match self.dir {
            Right => { self.col = if self.col == WIDTH { 0 } else { self.col + 1 }; },
            Left => { self.col = if self.col == 0 { WIDTH } else { self.col - 1 }; },
            Up => { self.line = if self.line == 0 { HEIGHT } else { self.line - 1 }; },
            Down => { self.line = if self.col == HEIGHT { 0 } else { self.line + 1 }; }
        }
    }

    fn at(&mut self, line: i32, col: i32) -> &mut i32 {
        let line_wrap = usize::try_from(line.rem_euclid(HEIGHT as i32)).unwrap();
        let col_wrap = usize::try_from(col.rem_euclid(HEIGHT as i32)).unwrap();
        &mut self.playfield[line_wrap][col_wrap]
    } 
    
    fn step(&mut self, stdin: &mut io::StdinLock, stdout: &mut io::StdoutLock) -> Result<bool,Error> {
        use Direction::*;
        use Error::*;

        let instr = self.playfield[self.line][self.col];

        if self.str_mode {
            if instr == 34 {
                self.str_mode = false;
            } else {
                self.push(instr);
            }
        } else {
            match instr {
                48 ..= 57 => { self.push(instr - 48); },
                42 => { // *
                    let a = self.pop();
                    let b = self.pop();
                    self.push(b.wrapping_mul(a));
                },
                43 => { // +
                    let a = self.pop();
                    let b = self.pop();
                    self.push(b.wrapping_add(a));
                },
                45 => { // -
                    let a = self.pop();
                    let b = self.pop();
                    self.push(b.wrapping_sub(a));
                },
                47 => { // /
                    let a = self.pop();
                    let b = self.pop();
                    self.push(b / a);
                },
                37 => { // /
                    let a = self.pop();
                    let b = self.pop();
                    self.push(b % a);
                },
                33 => { // !
                    let a = self.pop();
                    self.push(if a == 0 { 1 } else { 0 });
                },
                96 => { // `
                    let a = self.pop();
                    let b = self.pop();
                    self.push(if b > a { 1 } else { 0 });
                },
                62 => { // >
                    self.dir = Right;
                },
                60 => { // <
                    self.dir = Left;
                },
                94 => { // ^
                    self.dir = Up;
                },
                118 => { // v
                    self.dir = Down;
                },
                63 => { // ?
                    self.dir = rand::random();
                },
                95 => { // _
                    self.dir = if self.pop() == 0 { Right } else { Left };
                },
                124 => { // |
                    self.dir = if self.pop() == 0 { Down } else { Up };
                },
                34 => { // "
                    self.str_mode = true;
                },
                58 => { // :
                    let a = self.pop();
                    self.push(a);
                    self.push(a);
                },
                92 => { // \
                    let a = self.pop();
                    let b = self.pop();
                    self.push(a);
                    self.push(b);
                },
                36 => { // $
                    self.pop();
                },
                35 => { // #
                    self.mov();
                },
                112 => { // p
                    let line = self.pop();
                    let col = self.pop();
                    let a = self.pop();
                    *self.at(line,col) = a;
                },
                103 => { // g
                    let line = self.pop();
                    let col = self.pop();
                    let val = *self.at(line,col);
                    self.push(val);
                },
                64 => { // @
                    return Ok(false);
                },
                126 => { // ~
                    let mut buf: [u8;1] = [0];
                    let mut res: io::Result<usize>;
                    loop {
                        res = stdin.read(&mut buf);
                        if res.is_ok() {
                            break;
                        }
                    }
                    self.push(buf[0] as i32);
                },
                38 => { // &
                    let mut buf = String::new();
                    let mut res: io::Result<usize>;
                    loop {
                        res = stdin.read_line(&mut buf);
                        if res.is_ok() {
                            break;
                        }
                    }
                    let inp: i32 = buf.trim().parse().map_err(|e| {ParseIntFailed(e)})?;
                    self.push(inp);
                },
                44 => { // ,
                    stdout.write_all(&self.pop().to_ne_bytes()).map_err(|e| {WriteFailed(e)})?;
                    stdout.flush().map_err(|e| {WriteFailed(e)})?;
                },
                46 => { // .
                    stdout.write_all(self.pop().to_string().as_bytes()).map_err(|e| {WriteFailed(e)})?;
                    stdout.write_all(&[32]).map_err(|e| {WriteFailed(e)})?;
                    stdout.flush().map_err(|e| {WriteFailed(e)})?;
                },
                _ => { }
            }
        }

        self.mov();

        Ok(true)
    }

    fn run(&mut self) -> Result<(),Error> {
        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        loop {
            if !self.step(&mut stdin, &mut stdout)? {
                return Ok(());
            }
        }
    }
}

fn main() -> Result<(),Error>{
    let filename = std::env::args().next_back().ok_or(Error::NoArgs)?;
    let source = std::fs::read_to_string(filename).map_err(|e| {Error::ReadSourceFailed(e)})?;
    let prog = parse_program(&source)?;
    State::init(prog).run()
}