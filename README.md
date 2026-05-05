# RustGuess - A Guessing Game Module in the Kernel

**rustguess** is a Linux kernel module that runs a number-guessing game at /dev/rustguess — a small example of stateful, user-driven protocols at Ring 0, written entirely in safe Rust. The state machine is mutex-protected; malformed input is handled with a polite error message rather than a kernel panic.

// What this is paragraph ^^^

## DEMO (Complete Later)


### Build-and-Run


## Design Notes


### Future Work
Random Secret - *Use the kernel RNG (kernel::random::getrandom) to pick a fresh SECRET at module load.*
Per-open game state - *Each open() of the device gets its own game — multiple users can play simultaneously without sharing a secret.Move the GameState out of the global lock and into the per-open RustGuessDevice.*
Difficulty levels - *Write RANGE:1000\n to expand the search space to 1–1000 before guessing.*
Hint history - *Track the user's guesses and return them on a special HISTORY\n write — show the player how they narrowed in on the answer.*
Cheat code - *A debug-mode REVEAL\n command that prints the secret.*
A /proc/rustguess view - *A read-only window that shows the secret to the kernel sysadmin (root) for testing.*

### License
Licensed GPL-2.0 to match the Linux kernel.