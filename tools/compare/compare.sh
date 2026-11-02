#!/bin/sh
# Compares this port with the C# original: painted pixels and seeded behaviour.
# Usage (Windows, Git Bash, from the repo root): tools/compare/compare.sh /path/to/original
set -e
ORIG=$(cd "$1" && pwd -W)
ROOT=$(pwd -W)
OUT=$ROOT/target/compare
mkdir -p "$OUT"
CSC=/c/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe
cargo build --release --bin hash --bin sim_trace
cd "$ORIG"
"$CSC" -nologo -nowarn:0168,0219,0414 -optimize+ -out:"$OUT/hash_cs.exe" -r:System.Drawing.dll -r:System.Windows.Forms.dll 'src\Domain\*.cs' 'src\Rendering\*.cs' 'src\Settings.cs' 'tools\Hash.cs'
"$CSC" -nologo -nowarn:0168,0219,0414 -optimize+ -out:"$OUT/sim_cs.exe" -r:System.Drawing.dll 'src\Domain\*.cs' 'src\Settings.cs' "$(cygpath -w "$ROOT/tools/compare/SimTrace.cs")"
cd "$ROOT"
"$OUT/hash_cs.exe" | tr -d '\r' > "$OUT/hash_cs.txt"
target/release/hash > "$OUT/hash_rs.txt"
"$OUT/sim_cs.exe" | tr -d '\r' > "$OUT/sim_cs.txt"
target/release/sim_trace > "$OUT/sim_rs.txt"
cmp "$OUT/hash_cs.txt" "$OUT/hash_rs.txt" && echo "painters: pixel-identical ($(wc -l < "$OUT/hash_rs.txt") poses)"
# Column 2 hashes raw doubles, whose last bits depend on the maths library: not compared.
strip() { awk '/^[0-9]/ { $2 = "" } { print }' "$1"; }
strip "$OUT/sim_cs.txt" > "$OUT/sim_cs.cmp"
strip "$OUT/sim_rs.txt" > "$OUT/sim_rs.cmp"
cmp "$OUT/sim_cs.cmp" "$OUT/sim_rs.cmp" && echo "simulation: identical ($(wc -l < "$OUT/sim_rs.txt") lines)"
