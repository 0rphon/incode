# incode

InCode: for encoding your code in code to run in other peoples code!\
\
I was inspired to create this after being disappointed in the results that !(Slinky.py)[https://github.com/ihack4falafel/Slink]'s generated. Considering Slinky seems to be the script of choice in a lot of blogs and forums, i figured i might as well upload my own alternative.\
\
right now all it contains is a tool to wrap non-ascii commands in ascii shellcode. just pass the bytes you want to incode as an argument. i made the regex as flexible on formatting as i could so as long as the bytes are somewhere in your input incode should be able to parse it alright. for now it still needs you to handle esp location\
example: incode.exe "\xE9\xFF\xC0\xB7\x30"\
\
This is a relatively new project but my goal is to add in any tools that i find myself needing over the course of future projects.
