@echo on
echo Starting QEMU...
echo Using QEMU executable: "C:\Program Files\QEMU\qemu-system-x86_64w.exe"
echo Disk image: target/x86_64-screamos/debug/bootimage-screamos.bin

if not exist "C:\Program Files\QEMU\qemu-system-x86_64w.exe" (
  echo ERROR: QEMU executable not found!
  pause
  exit /b 1
)

if not exist target/x86_64-screamos/debug/bootimage-screamos.bin (
  echo ERROR: Boot image not found!
  pause
  exit /b 1
)

"C:\Program Files\QEMU\qemu-system-x86_64w.exe" -drive format=raw,file=target/x86_64-screamos/debug/bootimage-screamos.bin -monitor stdio

echo QEMU returned with exit code: %ERRORLEVEL%
pause 