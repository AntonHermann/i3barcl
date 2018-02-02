extern crate ncurses;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate error_chain;

use ncurses::*;
use std::io::{self,Read};
use std::thread;
use std::time::Duration;
use errors::*;
use error_chain::ChainedError; // trait which holds `display_chain`

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    full_text: String,
    color: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Blocks(Vec<Block>);

fn main() {
    // init ncurses
    initscr();
    noecho();
    cbreak(); // do not line buffer
    timeout(0);

    if let Err(ref e) = run() {
        eprintln!("Error: {}", e.display_chain());
    }

    endwin();
}

fn run() -> Result<()> {
    let mut stdin = io::stdin();

    let mut buffer = [0; 2048];

    skip_until(&mut stdin, b'}');
    skip_until(&mut stdin, b'[');

    let mut error_count = 0;
    let mut errors: Vec<Error> = Vec::new();

    loop {
        if let Ok(_) = stdin.read(&mut buffer) {
            match Blocks::from_json(&buffer) {
                Ok(blocks) => {
                    error_count = 0;
                    mvprintw(0, 0, blocks.to_string().as_str());
                },
                Err(e) => {
                    error_count += 1;
                    errors.push(e);
                    // single errors are okay, but if something
                    // keeps going wrong, exit, printing all
                    // errors to stderr
                    if error_count >= 10 {
                        for error in errors {
                            eprintln!("Error: {}", error.display_chain());
                        }
                        bail!("10 errors in series");
                    }
                },
            }
        }
        thread::sleep(Duration::new(0,250_000_000));
        refresh();
    }
}

fn get_last_block(data: &[u8]) -> Result<&[u8]> {
    let mut end = data.len();
    while data[end-1] != b']' {
        if end == 0 {
            bail!("not enough data read to read block");
        }
        end -= 1;
    }
    let mut start = end;
    while data[start] != b'[' {
        if start == 0 {
            bail!("not enough data read to read block");
        }
        start -= 1;
    }
    Ok(&data[start..end])
}

impl Blocks {
    pub fn from_json(data: &[u8]) -> Result<Self> {
        let last_block = get_last_block(data)?;
        let blocks: Vec<Block> = serde_json::from_slice(last_block)
            .chain_err(|| "failed to parse blocks from json")?;
        Ok(Blocks(blocks))
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for b in &self.0 {
            out.push_str(b.full_text.as_str());
            out.push(' ');
        }
        out
    }
}

fn skip_until<R: Read>(reader: &mut R, ch: u8) {
    loop {
        let mut b = [0; 1];
        let _ = reader.read_exact(&mut b); // skip first message
        if b[0] == ch {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_comma() {
        assert_eq!(b"bla", Blocks::trim_comma(b"bla,"));
        assert_eq!(b"bla", Blocks::trim_comma(b",bla"));
        assert_eq!(b"bla", Blocks::trim_comma(b",bla,"));
        assert_eq!(b"bla", Blocks::trim_comma(b"bla"));
    }

    #[test]
    fn test_trim_trailing_zeroes() {
        assert_eq!([1,2,3], Blocks::trim_trailing_zeroes(&[1,2,3,0,0]));
        assert_eq!([1,2,3], Blocks::trim_trailing_zeroes(&[1,2,3]));
    }

    #[test]
    fn test_parse_blocks() {
        let block1 = Block {
            full_text: String::from("test"),
            color: String::from("black"),
        };
        let block2 = Block {
            full_text: String::from("block2"),
            color: String::from("red"),
        };
        let blocks = Blocks(vec![block1, block2]);
        let data = b"[{\"full_text\":\"test\",\"color\":\"black\"},{\"full_text\":\"block2\",\"color\":\"red\"}]";
        let mut data2 = [0; 1024];
        let mut data3 = [0; 1024];
        data2[..data.len()].clone_from_slice(data);
        data3[..data.len()].clone_from_slice(data);
        data3[data.len()] = b',';

        assert_eq!(blocks.to_string(), Blocks::from_json(data).to_string());
        assert_eq!(blocks.to_string(), Blocks::from_json(&data2).to_string());
        assert_eq!(blocks.to_string(), Blocks::from_json(&data3).to_string());
    }
}
