@echo off
"C:\Program Files\qemu\qemu-system-x86_64w.exe" -drive format=raw,file=target\x86_64-screamos\debug\bootimage-screamos.bin -no-reboot -no-shutdown

::  Alternativ:
::  -m 256M = tilldelar 256MB RAM till den virtuella maskinen
::  -cpu host = använder värdmaskinens CPU-funktioner
::  -smp 2 = använder 2 virtuella CPUer
::  -serial stdio = dirigerar seriell output till konsolen
::  -no-reboot = avslutar QEMU vid omstart istället för att starta om
::  -no-shutdown = avslutar inte QEMU vid avstängning 