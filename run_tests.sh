#!/bin/sh

INPUT_VERSION=1.2.3 INPUT_FILE_PATH=./tests/test.csproj ./target/release/version-bump                 ─╯
INPUT_VERSION=1.2.3 INPUT_FILE_PATH=./tests/AndroidManifest.xml ./target/release/version-bump         ─╯
INPUT_VERSION=1.2.3 INPUT_FILE_PATH=./tests/Info.plist ./target/release/version-bump                  ─╯
INPUT_VERSION=1.2.3 INPUT_FILE_PATH=./tests/dir.build.props ./target/release/version-bump             ─╯
INPUT_VERSION=1.2.3 INPUT_FILE_PATH=./tests/package-lock.json ./target/release/version-bump           ─╯

