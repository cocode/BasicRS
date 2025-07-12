#  <img src="images/logo2.png" alt="Logo" width="50" height="25"> BasicRS

BasicRS is a Rust implementation of a BASIC Interpreter 

It's part of a family of BASIC interpreters that started with 
TrekBasic, written in Python, and TrekBasicJ, written in Java.

TrekBasic was hand-written, but BasicRS was mostly translated by AI
from TrekBasic.

My goal was to be able to play the old Star Trek game, which was written in BASIC.

    https://en.wikipedia.org/wiki/Star_Trek_(1971_video_game). 

I have achieved that goal.

## To Run

cargo run -- superstartrek.bas

OR

target/debug/basic_rs superstartrek.bas

## Shell
If you want to use the shell for BASICk which is the command line "IDE" - sort of.

It provides breakpoints, single stepping, code coverage, and more.  

./target/debug/basic_shell superstartrek.bas

or just

./target/debug/basic_shell
