#!/bin/bash
set -euo pipefail

# Create a basic retry wrapper
# Use this for anything that makes external network requests
# This way a single erroneuous network failure doesn't fail the whole build
retry() {
	for i in {1..10}; do
		if [ $i -gt 1 ]; then
			echo "Retrying... (attempt $i/10)"
		fi

		$@ || {
			continue
		}

		if [[ "$?" == "0" ]]; then
			return 0
		fi

		if [ $i -eq 10 ]; then
			echo "The operation failed after 10 attempts"
			return 1
		fi
	done
}

# Define the arch we're building for
# In the future, to build for aarch64, we can re-run all these commands and just change the arch here
# We would just need to set up an aarch64 sysroot first
export LINUX_ARCH="aarch64"


  export LDFLAGS="-L/opt/homebrew/opt/openssl@3/lib"
  export CPPFLAGS="-I/opt/homebrew/opt/openssl@3/include"
  export OPENSSL_ROOT_DIR="/opt/homebrew/opt/openssl@3"
  export OPENSSL_INCLUDE_DIR="/opt/homebrew/opt/openssl@3/include"
  export OPENSSL_LIB_DIR="/opt/homebrew/opt/openssl@3/lib"

  export CARGO_CMAKE_ARGS="-DOPENSSL_ROOT_DIR=/opt/homebrew/opt/openssl@3 -DOPENSSL_INCLUDE_DIR=/opt/homebrew/opt/openssl@3/include -DOPENSSL_LIB_DIR=/opt/homebrew/opt/openssl@3/lib"

# Build tango
# CARGO_CMAKE_ARGS="-DOPENSSL_ROOT_DIR=/opt/homebrew/opt/openssl@3 -DOPENSSL_LIB_DIR=/opt/homebrew/opt/openssl@3/lib -DOPENSSL_INCLUDE_DIR=/opt/homebrew/opt/openssl@3/lib/include" \
# cargo apk build \
# 	# --build-arg OPENSSL_ROOT_DIR=/opt/homebrew/opt/openssl@3 \
# 	-p tango \
# 	--no-default-features \
# 	--features=sdl2-audio,wgpu,cpal \
# 	--release 

# echo "Building Rust App..."
# export OPENSSL_ROOT_DIR=/opt/homebrew/opt/openssl@3
# export OPENSSL_INCLUDE_DIR=/opt/homebrew/opt/openssl@3/include
# export OPENSSL_LIB_DIR=/opt/homebrew/opt/openssl@3/lib
# export CARGO_CMAKE_ARGS="-DOPENSSL_ROOT_DIR=/opt/homebrew/opt/openssl@3 -DOPENSSL_LIB_DIR=/opt/homebrew/opt/openssl@3/lib -DOPENSSL_INCLUDE_DIR=/opt/homebrew/opt/openssl@3/include"

# OPENSSL_ROOT_DIR=/opt/homebrew/opt/openssl@3 
# OPENSSL_INCLUDE_DIR=$OPENSSL_ROOT_DIR/include 

# CARGO_CMAKE_ARGS="-DOPENSSL_ROOT_DIR=/opt/homebrew/opt/openssl@3 -DOPENSSL_LIB_DIR=/opt/homebrew/opt/openssl@3/lib -DOPENSSL_INCLUDE_DIR=/opt/homebrew/opt/openssl@3/include" \
AR=llvm-ar \
PATH="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin:$PATH" \
AR=llvm-ar cargo build \
	--release \
	--target=aarch64-linux-android \
	--bin tango \
	--no-default-features \
	--features=sdl2-audio,wgpu,cpal

	# cargo apk build \
	# -p tango \
	# --no-default-features \
	# --features=sdl2-audio,wgpu,cpal \
	# --release 
