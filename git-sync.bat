@echo off
set MESSAGE=%1
if "%MESSAGE%"=="" set MESSAGE=Update codebase

git add .
git commit -m "%MESSAGE%"
git push
