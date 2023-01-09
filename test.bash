#!/usr/bin/env bash

printf "tests for valid denvl.\n"
find ./example-codes/valid/ -type f | while read line; do
  printf "  $line ... "
  cargo run -- run $line > /dev/null 2>&1
  if [[ $? -eq 0 ]]; then
    printf "\033[32mok\033[m\n"
  else
    printf "\033[31mfailed\033[m\n"
  fi
done

printf "tests for invalid denvl.\n"
find ./example-codes/invalid/ -type f | while read line; do
  printf "  $line ... "
  cargo run -- run $line > /dev/null 2>&1
  if [[ $? -eq 0 ]]; then
    printf "\033[31mfailed\033[m\n"
  else
    printf "\033[32mok\033[m\n"
  fi
done

