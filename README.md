# incode

InCode: for encoding your code in code to run in other peoples code!\
\
I was inspired to create this after being disappointed in the results that [Slinky.py](https://github.com/ihack4falafel/Slink) generated. Considering Slinky seems to be the script of choice in a lot of blogs and forums, I figured I might as well upload my own alternative. Right now all it contains is a tool to wrap non-ascii commands in ascii shellcode via add/sub commands with checks for if it should xor beforehand.\
\
Just pass the bytes you want to incode as an argument. I made the regex as flexible as i could so, as long as the bytes are somewhere in your input, incode should be able to parse it. for now it still needs you to handle adjusting esp location but thats my next step.\
example: `incode.exe "\xE9\xFF\xC0\xB7\x30"`\
\
This is a relatively new project but my goal is to add in any tools that I find myself consistantly needing over the course of future projects.\
My current goals are:
 - Generate code to adjust ESP
 - Generate code to far jump
 - Add xor/and instructions to the ascii wrapper to try and increase efficency
