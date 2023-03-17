$sourcePath = "C:\Users\kaller\dev\rustgame\src\main.rs"
$buildCommand = "cargo build --target wasm32-unknown-unknown --release"
$targetPath = "C:\Users\kaller\dev\rustgame\target\wasm32-unknown-unknown\release\rustgame.wasm"
$destinationPath = "C:\Users\kaller\dev\rustgamewasmhost\public\rustgame.wasm"

# Change the current directory to the source folder
Set-Location (Split-Path $sourcePath)

# Run the build command
Invoke-Expression $buildCommand

# Move the wasm file to the destination folder
Move-Item $targetPath $destinationPath -Force
