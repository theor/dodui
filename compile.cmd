@echo off
set FXC="C:\Program Files (x86)\Windows Kits\10\bin\10.0.17763.0\x64\fxc.exe" -nologo
if not exist data mkdir data
%FXC% /T vs_4_0 /E Vertex /Fo data/vertex.fx shader/cube.hlsl
%FXC% /T ps_4_0 /E Pixel /Fo data/pixel.fx shader/cube.hlsl
