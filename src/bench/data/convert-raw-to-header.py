#!/usr/bin/env python3
# Copyright (c) 2019 The Bitcoin developers

import sys


def main(test_name, input_file):
    with open(input_file, "rb") as f:
        contents = f.read()

    print(f"static unsigned const char {test_name}_raw[] = {{")
    print(", ".join(map(lambda x: f"0x{x:02x}", contents)))
    print("};")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("We need additional pylons!")
        sys.exit(1)

    main(sys.argv[1], sys.argv[2])
