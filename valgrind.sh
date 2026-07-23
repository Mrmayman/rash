#!/bin/env bash
cargo build
valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --gen-suppressions=all --suppressions=valgrind.supp target/debug/rash
