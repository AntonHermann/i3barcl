extern crate ncurses;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate error_chain;

use ncurses::*;
use std::io::Read;
use std::process::{Command, Stdio};
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

static CMD: &'static str = "/home/anton/code/rust/i3status-rust/target/release/i3status-rs";
static ARG: &'static str = "/home/anton/.config/i3status/config.toml";

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
    let mut monitor = Command::new(CMD)
        .arg(ARG)
        .stdout(Stdio::piped())
        .spawn()
        .chain_err(|| "Failed to start programm")?
        .stdout
        .chain_err(|| "Failed to pipe program output")?;

    let mut buffer = [0; 2048];

    let mut ch;
    let _ = monitor.read(&mut buffer); // skip first message
    let _ = monitor.read_exact(&mut [0; 1]);
    loop {
        ch = getch();
        if ch != ERR {
            break Ok(());
        }

        if let Ok(_) = monitor.read(&mut buffer) {
            let blocks = Blocks::from_json(&buffer)?;
            mvprintw(0, 0, blocks.to_string().as_str());
        }
        thread::sleep(Duration::new(0,250_000_000));
        refresh();
    }
}

impl Blocks {
    pub fn from_json(data: &[u8]) -> Result<Self> {
        let sanitized = Blocks::trim_comma(Blocks::trim_trailing_zeroes(data));
        let blocks: Vec<Block> = serde_json::from_slice(sanitized)
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

    fn trim_comma(arr: &[u8]) -> &[u8] {
        let last_id = arr.len() - 1;
        let start = if arr[0] == b',' { 1 } else { 0 };
        let end = if arr[last_id] == b',' { last_id } else { last_id + 1 };

        &arr[start..end]
    }

    fn trim_trailing_zeroes(arr: &[u8]) -> &[u8] {
        for i in (0..arr.len()).rev() {
            if arr[i] != 0 {
                return &arr[..i+1];
            }
        }
        return &arr[0..0];
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
