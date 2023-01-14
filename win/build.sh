#!/bin/bash
set -euo pipefail

# Cleanup.
function cleanup {
    rm -rf Tango.iconset tango_win_workdir
}
trap cleanup EXIT
cleanup

# Generate an appropriate .rc file.
"$(dirname "${BASH_SOURCE[0]}")/generate_rc.py" >tango/resource.rc

# Create icon.
# mkdir Tango.iconset
# convert -resize 16x16 tango/src/icon.png -depth 32 Tango.iconset/icon_16x16.png
# convert -resize 32x32 tango/src/icon.png -depth 32 Tango.iconset/icon_32x32.png
# convert -resize 128x128 tango/src/icon.png -depth 32 Tango.iconset/icon_128x128.png
# convert -resize 256x256 tango/src/icon.png -depth 32 Tango.iconset/icon_256x256.png
# convert Tango.iconset/*.png tango/icon.ico
# rm -rf Tango.iconset

# Build Windows binaries.
cargo build --bin tango --release --target x86_64-pc-windows-gnu

# Build installer.
mkdir tango_win_workdir
"$(dirname "${BASH_SOURCE[0]}")/generate_nsi.py" >tango_win_workdir/installer.nsi
pushd tango_win_workdir

cp ../tango/icon.ico .
cp ../target/x86_64-pc-windows-gnu/release/tango.exe .
cp {/usr/x86_64-w64-mingw32/lib/libwinpthread-1.dll,/usr/lib/gcc/x86_64-w64-mingw32/10-posix/{libgcc_s_seh-1.dll,libstdc++-6.dll}} .

ANGLE_ZIP_URL="https://github.com/google/gfbuild-angle/releases/download/github%2Fgoogle%2Fgfbuild-angle%2Ff810e998993290f049bbdad4fae975e4867100ad/gfbuild-angle-f810e998993290f049bbdad4fae975e4867100ad-Windows_x64_Release.zip"
mkdir angle
wget -O - "${ANGLE_ZIP_URL}" | bsdtar -Cangle -xvf- lib/{libEGL.dll,libGLESv2.dll}
cp angle/lib/{libEGL.dll,libGLESv2.dll} .

FFMPEG="https://github.com/eugeneware/ffmpeg-static/releases/download/b5.0.1/win32-x64"
wget -O ffmpeg.exe "${FFMPEG}"

makensis installer.nsi
popd

mkdir -p dist
mv tango_win_workdir/installer.exe "dist/tango-x86_64-windows.exe"
rm -rf tango_win_workdir
