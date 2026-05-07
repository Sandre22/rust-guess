# RustGuess - A Guessing Game Module in the Kernel

**rustguess** is a Linux kernel module that runs a number-guessing game at /dev/rustguess — a small example of stateful, user-driven protocols at Ring 0, written entirely in safe Rust. The state machine is mutex-protected; malformed input is handled with a polite error message rather than a kernel panic.

## DEMO (Complete Later)
```
make clean && make
sudo insmod rustguess.ko
ls -la /dev/rustguess

sudo cat /dev/rustguess                       # welcome message
echo 50 | sudo tee /dev/rustguess > /dev/null
sudo cat /dev/rustguess                       # "50 is too high -- guess lower."
echo 25 | sudo tee /dev/rustguess > /dev/null
sudo cat /dev/rustguess                       # "25 is too low -- guess higher."
echo 42 | sudo tee /dev/rustguess > /dev/null
sudo cat /dev/rustguess                       # "Correct! You got it in 3 tries."
echo 50 | sudo tee /dev/rustguess > /dev/null
sudo cat /dev/rustguess                       # "You already won!..."

sudo rmmod rustguess
sudo insmod rustguess.ko                      # new game (still secret 42 in this build)
sudo cat /dev/rustguess                       # welcome message again
sudo rmmod rustguess
```

### Build-and-Run


## Design Notes
RustGuess stores all game states in a single Mutex<GameState> field on the RustGuessDevice struct, which is registered as a misc device at module load time via miscdev::Registration::new_pinned. Because the device is a singleton and the mutex lives on the device struct directly, every open of RustGuess shares the same in-progress game.

The write handler copies bytes from user space with data.read_all, parses them as a UTF-8 using core::str::from_utf8, and hands the result to Rust's built in .parse::<u64>(). This allows unacceptable inputs such as letters or float points to be handled with a friendly error message rather than a kernel panic.

A won flag makes the device modal: once the correct answer is submitted, all subsequent write commands default to a "You already won!" message, keeping the game state coherent without any additional logic.

The consumed flag on GameState implements a simple EOF protocol: read returns the latest message once and then returns 0 for every subsequent call until a new write clears the flag, which lets cat exit cleanly instead of looping.

The secret is hardcoded at compile time (SECRET = 42) for simplicity; replacing this code with a kernel RNG call at init time would be a natural extension.

### Code Tour
If you want to read the code, start at *init* (line 40), this is where our GameState is set up as well as registration of the RustGuessDevice.

Next, look into *write* (line 67) where we parse the input and update GameState.

Then, go to *read* (line 111) where we return the GameState.

Finally, end with *drop* (line 59) which cleans up and effectively ends our game.

### Future Work
Random Secret - *Use the kernel RNG (kernel::random::getrandom) to pick a fresh SECRET at module load.*

Per-open game state - *Each open() of the device gets its own game — multiple users can play simultaneously without sharing a secret. Move the GameState out of the global lock and into the per-open RustGuessDevice.*

Difficulty levels - *Write RANGE:1000\n to expand the search space to 1–1000 before guessing.*

Hint history - *Track the user's guesses and return them on a special HISTORY\n write — show the player how they narrowed in on the answer.*

Cheat code - *A debug-mode REVEAL\n command that prints the secret.*

A /proc/rustguess view - *A read-only window that shows the secret to the kernel sysadmin (root) for testing.*

#### License
Licensed GPL-2.0 to match the Linux kernel.