mod code_gen;

use clap::{Parser, ValueEnum};
use inkwell::module::Module;
use std::task::Context;
use std::{collections::VecDeque, fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Block {
    Block {
        delta: Vec<u8>,
        delta_start: i32,
        delta_ptr: i32,
    },
    Input,
    Output,
    LoopOpen(u32),
    LoopClose(u32),
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Token {
    Inc,
    Dec,
    Opn,
    Cls,
    Lft,
    Rgh,
    Out,
    Inp,
}

fn parse_source(source: String) -> Vec<Token> {
    source
        .chars()
        .filter_map(|x| match x {
            '+' => Some(Token::Inc),
            '-' => Some(Token::Dec),
            '[' => Some(Token::Opn),
            ']' => Some(Token::Cls),
            '>' => Some(Token::Rgh),
            '<' => Some(Token::Lft),
            '.' => Some(Token::Out),
            ',' => Some(Token::Inp),
            _ => None,
        })
        .collect()
}

fn run_block(block: &Vec<Token>) -> Block {
    if block == &vec![Token::Lft] {
        return Block::Left;
    } else if block == &vec![Token::Rgh] {
        return Block::Right;
    }

    let mut buff = VecDeque::from(block.clone());
    let mut delta_ptr = 0;
    let mut curr_ptr = 0;
    let mut delta = VecDeque::<i128>::from([0]);
    let mut delta_start = 0;

    while let Some(next_elm) = buff.pop_front() {
        match next_elm {
            Token::Inc => {
                delta.get_mut(curr_ptr).map(|x| {
                    *x += 1;
                });
            }
            Token::Dec => {
                delta.get_mut(curr_ptr).map(|x| {
                    *x -= 1;
                });
            }
            Token::Lft => {
                delta_ptr -= 1;

                if curr_ptr == 0 {
                    delta.push_front(0);
                } else {
                    curr_ptr -= 1;
                }

                if delta_ptr < delta_start {
                    delta_start = delta_ptr;
                }
            }
            Token::Rgh => {
                delta_ptr += 1;
                curr_ptr += 1;

                if curr_ptr >= delta.len() {
                    delta.push_back(0);
                }
            }

            _ => unreachable!(),
        }
    }

    Block::Block {
        delta: delta.into(),
        delta_start,
        delta_ptr,
    }
}

fn mk_block(source: Vec<Token>) {
    let mut queue: VecDeque<Token> = source.into();
    let mut blocks = vec![];
    let mut next_block = vec![];

    let mut max = 1;
    let mut loop_number = vec![];

    while let Some(token) = queue.pop_front() {
        match token {
            Token::Inp => {
                blocks.push(Block::Input);

                blocks.push(run_block(&next_block));
                next_block.clear();
            }
            Token::Out => {
                blocks.push(Block::Output);

                blocks.push(run_block(&next_block));
                next_block.clear();
            }
            Token::Opn => {
                blocks.push(Block::LoopOpen(max));
                loop_number.push(max);
                blocks.push(run_block(&next_block));
                max += 1;
            }
            Token::Cls => {
                blocks.push(Block::LoopOpen(
                    loop_number.pop().expect("Found more ] then ["),
                ));
                loop_number.push(max);
            }
            _ => next_block.push(token),
        }
    }
}

fn main() {
    let args = Args::parse();

    println!("{args:?}");

    let source = fs::read_to_string(&args.file).expect("Could not open source file");

    let parsed = parse_source(source);

    let blocks = mk_block(parsed);

    // let ctx = inkwell::context::Context::create();
    // let module = ctx.create_module(
    //     args.file
    //         .file_name()
    //         .expect("Invalid filename")
    //         .to_str()
    //         .expect("Invalid filename"),
    // );
    // let builder = ctx.create_builder();

    // let memory = create_memory_u8(&ctx, module);
    // let set_start = create_start_fn();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser() {
        let sample = parse_source("+-<>.,[]".to_string());
        let expected = vec![
            Token::Inc,
            Token::Dec,
            Token::Lft,
            Token::Rgh,
            Token::Out,
            Token::Inp,
            Token::Opn,
            Token::Cls,
        ];
        assert_eq!(sample, expected);
    }

    #[test]
    fn test_run_block() {
        let ex = parse_source("->+>+<<".to_string());
        let block_repr = run_block(&ex);
        let expected = Block::Block {
            delta: vec![-1, 1, 1],
            delta_start: 0,
            delta_ptr: 0,
        };
        assert_eq!(block_repr, expected);

        let ex = parse_source("++<-".to_string());
        let block_repr = run_block(&ex);
        let expected = Block::Block {
            delta: vec![-1, 2],
            delta_start: -1,
            delta_ptr: -1,
        };
        assert_eq!(block_repr, expected);

        let ex = parse_source("++<->->+".to_string());
        let block_repr = run_block(&ex);
        let expected = Block::Block {
            delta: vec![-1, 1, 1],
            delta_start: -1,
            delta_ptr: 1,
        };
        assert_eq!(block_repr, expected);
    }

    #[test]
    fn test_mk_block() {
        let source_code = parse_source("+++[->+>+<<].>.>.".to_string());
        let parsed = mk_block(source_code);
        let expected = vec![
            Block::Block {
                delta: vec![3],
                delta_start: 0,
                delta_ptr: 0,
            },
            Block::LoopOpen(1),
            Block::Block {
                delta: vec![-1, 1, 1],
                delta_start: 0,
                delta_ptr: 0,
            },
            Block::Output,
            Block::Left,
            Block::Output,
            Block::Left,
            Block::Output,
            Block::Left,
        ];
    }
}
