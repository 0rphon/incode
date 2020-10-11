# incode

InCode: for encoding your code in code to run in other peoples code!\
\
Incode is a tool for wrapping x86 shellcode in ascii safe commands. It contains tools to help with wrapping, unpacking, and jumping in ascii restricted payloads.\
\
This is a relatively new project but my goal is to add in any tools that I find myself consistently needing over the course of future projects.\
My current goals are:
- Generate code to far jump
- Add xor/and instructions to the ascii wrapper to try and increase efficiency

```None
InCode is an ASCII encoder for x86 shellcode. It has tools to handle wrapping, positioning, and jumping.
This is a tool I wrote for personal security research. I obviously accept no responsibility for how other
people use it.

Usage:
    --wrap [bytes]:     Wrap an instruction in x86 ascii shellcode that gets decoded to [esp] at runtime.

    --esp [addr] --eip [addr]:
                        What the addresses of ESP and EIP will be at the first byte of this payload. While 
                        these values wont stay the same between runs, their difference will. Use this if 
                        you want the payload to handle unpacking on its own.

    --jump [addr]:      Generate a wrapped far jump from [eip] to [addr] that gets decoded to [esp] at 
                        runtime. Requires --esp and --eip to be set. It will handle esp positioning for 
                        you.
    
    --size              Use bruteforce to optimize size. Significantly slower but can sometimes save
                        room.

    --help:             Show this screen and exit.

Examples:
    Generate ASCII wrapped payload that decodes given values in memory:
        incode.exe \\xF3\\xE9\\xB8\\x00\\x33\\x4A\\x41

    Generate shellcode to position esp at your location:                                  
        incode.exe --esp 45D308 --eip 457B00

    Generate [positioning code]+[wrapped payload]:                                        
        incode.exe --wrap F3E9B800334A41 --esp 45D308 --eip 457B00

    (UNIMPLEMENTED) Generate [positioning code]+[wrapped far jump]:                                       
        incode.exe --jump 463303 --esp 45D308 --eip 457B00

    (UNIMPLEMENTED) Generate [positioning code]+[wrapped payload]+[wrapped far jump]:                       
        incode.exe --wrap \"0xF3 0xE9 0xB8 0x00 0x33 0x4A 0x41\" --jump 463303 --esp 45D308 --eip 457B00"
```
