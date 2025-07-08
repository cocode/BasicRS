# BasicRS

BasicRS is a Rust implementation of a BASIC Interpreter 

It's part of a family of BASIC interpreters that started with 
TrekBasic, written in Python, and TrekBasicJ, written in Java.

TrekBasic was hand-written, but BasicRS was mostly translated by AI
from TrekBasic. 

BasicRS is not yet completed. It runs, but has many implementation
errors to be fixed.

My goal was to be able to play the old Star Trek game, which was written in BASIC.

    https://en.wikipedia.org/wiki/Star_Trek_(1971_video_game). 

I have achieved that goal.

## To Run

cargo run -- test_suite/hello.bas


## Terminology
A LINE is made up of multiple STATEMENTS, each one beginning with a KEYWORD.

### LINE
    100 PRINT X:GOTO 200
### STATEMENTS
    "PRINT X" and "GOTO 100"
### KEYWORDS
    "PRINT", and "GOTO"

