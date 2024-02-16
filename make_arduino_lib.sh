#!/bin/bash
set -euxo pipefail

cd "${0%/*}"
PKG_ROOT_DIR=`pwd`

ARDUIDE_ARCH_DIR=cortex-m4/fpv4-sp-d16-hard
RUST_ARCH_NAME=thumbv7em-none-eabihf

ARDUIDE_LIB_DIR=./prog/libraries/greaheisl_lib
LIB_SKELETON_DIR=./arduino_lib_skeleton
WORKSPACE_DIR=./lib_rs

cd $WORKSPACE_DIR/greaheisl_lib
cargo build --release --target $RUST_ARCH_NAME --no-default-features
cbindgen --config cbindgen.toml --crate greaheisl_lib --output ../target/generated/greaheisl_lib.h
cd $PKG_ROOT_DIR

mkdir -p $ARDUIDE_LIB_DIR
rm -r $ARDUIDE_LIB_DIR/ 

cp -r $LIB_SKELETON_DIR/ $ARDUIDE_LIB_DIR
mkdir -p $ARDUIDE_LIB_DIR/src/$ARDUIDE_ARCH_DIR
cp $WORKSPACE_DIR/target/$RUST_ARCH_NAME/release/libgreaheisl_lib.a \
   $ARDUIDE_LIB_DIR/src/$ARDUIDE_ARCH_DIR/
cp $WORKSPACE_DIR/target/generated/*.h \
   $ARDUIDE_LIB_DIR/src/

