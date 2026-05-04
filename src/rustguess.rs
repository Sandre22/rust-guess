// SPDX-License-Identifier: GPL-2.0

use kernel::prelude::*;
use kernel::str::CString;
use kernel::sync::Mutex;
use kernel::{file, miscdev};

module! {
    type: RustGuess,
    name: "rustguess",
    author: "Skyler Andrews",
    description: "A number-guessing game in the kernel",
    license: "GPL",
}

const SECRET: u64 = 42;
const MAX_GUESS: u64 = 100;

struct GameState {
    last_message: Vec<u8>,
    consumed: bool,
    tries: u64,
    won: bool,
}

struct RustGuessDevice {
    state: Mutex<GameState>,
}

struct RustGuess {
    _dev: Pin<Box<miscdev::Registration<RustGuessDevice>>>,
}

fn build_message(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut v = Vec::new();
    v.try_extend_from_slice(bytes)?;
    Ok(v)
}

impl kernel::Module for RustGuess {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("rustguess: module loaded (secret picked, get guessing!)\n");
        let initial = build_message(
            b"Welcome! Guess a number between 1 and 100. `echo N > /dev/rustguess`, then `cat /dev/rustguess`.\n"
        )?;
        let dev = RustGuessDevice {
            state: Mutex::new(GameState {
                last_message: initial,
                consumed: false,
                tries: 0,
                won: false,
            }),
        };
        let reg = miscdev::Registration::new_pinned(fmt!("rustguess"), dev)?;
        Ok(Self { _dev: reg })
    }
}

impl Drop for RustGuess {
    fn drop(&mut self) {
        pr_info!("rustguess: module unloaded (game ended)\n");
    }
}

#[vtable]
impl file::Operations for RustGuessDevice {
    fn write(this: &Self, _file: &file::File, data: &mut impl IoBufferReader) -> Result<usize> {
        let len = data.len();
        let mut buf = Vec::new();
        buf.try_reserve(len)?;
        data.read_all(&mut buf)?;

        let s = core::str::from_utf8(&buf).unwrap_or("");
        let guess: Option<u64> = s.trim().parse().ok();

        let mut state = this.state.lock();

        if state.won {
            state.last_message = build_message(
                b"You already won! `rmmod rustguess && insmod rustguess.ko` to play again.\n"
            )?;
            state.consumed = false;
            return Ok(len);
        }

        let response_bytes = match guess {
            None => build_message(b"Couldn't parse your input as a number. Try again.\n")?,
            Some(g) if g == 0 || g > MAX_GUESS => build_message(
                b"Out of range — pick a number between 1 and 100.\n"
            )?,
            Some(g) => {
                state.tries += 1;
                let formatted = if g < SECRET {
                    CString::try_from_fmt(fmt!("{} is too low — guess higher.\n", g))?
                } else if g > SECRET {
                    CString::try_from_fmt(fmt!("{} is too high — guess lower.\n", g))?
                } else {
                    state.won = true;
                    CString::try_from_fmt(fmt!("Correct! You got it in {} tries.\n", state.tries))?
                };
                build_message(formatted.as_bytes())?
            }
        };

        state.last_message = response_bytes;
        state.consumed = false;

        pr_info!("rustguess: tries={}, won={}\n", state.tries, state.won);
        Ok(len)
    }
}

fn read(this: &Self, _file: &file::File, data: &mut impl IoBufferWriter) -> Result<usize> {
    let mut state = this.state.lock();

    if state.consumed {
        return Ok(0);
    }

    let to_write = core::cmp::min(data.len(), state.last_message.len());
    data.write_slice(&state.last_message[..to_write])?;
    state.consumed = true;
    Ok(to_write)
}