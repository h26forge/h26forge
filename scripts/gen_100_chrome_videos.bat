@echo off

setlocal enableDelayedExpansion

rem %RANDOM% returns a value between 0 and 32767
rem We sample three, do some ax+b, then truncate to 10 digits.

set rand1=%RANDOM%
set rand2=%RANDOM%
set rand3=%RANDOM%

set /a uniq_id=%rand1% * %rand2% + %rand3%

set "uniq_id=00000000000000000%uniq_id%"
set "uniq_id=!uniq_id:~-10!%"

if not exist "tmp" mkdir tmp

set output_dir=tmp\rand_100_vids_%uniq_id%
set output_log=%output_dir%\rand_100.log
set tool_args=--mp4 --mp4-rand-size --safestart
set generation_args=--ignore-edge-intra-pred --ignore-ipcm --config config/chrome.json --small
set RUST_BACKTRACE=1

if not exist "%output_dir%" mkdir %output_dir%

for /l %%i in (0, 1, 99) do (
   set "x=000%%i"
   set "x=!x:~-4!%"
   set "h26forge=.\h26forge.exe %tool_args% generate %generation_args% -o %output_dir%\video_!x!.264"
   echo !h26forge!
   call !h26forge! >> %output_log%
)

copy scripts\play_videos.html %output_dir%

echo "Log saved to %output_log%"
echo "Open %output_dir%\play_videos.html in the browser to play the videos"