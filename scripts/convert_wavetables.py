#!/usr/bin/env python3
"""
Convert Odin 2 C++ wavetable data to Rust const arrays.

This script parses the WavetableData.cpp file and generates Rust code.
"""

import re
import os
import sys

# Constants from Odin 2
NUMBER_OF_WAVETABLES = 160
SUBTABLES_PER_WAVETABLE = 33
WAVETABLE_LENGTH = 512

def parse_wavetable_cpp(cpp_file_path):
    """Parse the C++ wavetable data file and extract float values."""

    print(f"Reading {cpp_file_path}...")

    with open(cpp_file_path, 'r') as f:
        content = f.read()

    # Find the start of the data array
    match = re.search(r'wavetable_data\[.*?\]\[.*?\]\[.*?\]\s*=\s*\{', content)
    if not match:
        print("ERROR: Could not find wavetable_data array")
        return None

    # Extract all float values using regex
    # Match numbers like: 0.123, -0.456, 1.22465e-16, etc.
    float_pattern = r'-?\d+\.?\d*(?:e[+-]?\d+)?'

    # Start after the opening brace
    data_start = match.end()
    data_content = content[data_start:]

    # Find all floats
    floats = re.findall(float_pattern, data_content)

    # Convert to Python floats
    values = []
    for f in floats:
        try:
            values.append(float(f))
        except ValueError:
            continue

    expected_count = NUMBER_OF_WAVETABLES * SUBTABLES_PER_WAVETABLE * WAVETABLE_LENGTH
    print(f"Found {len(values)} float values (expected {expected_count})")

    if len(values) < expected_count:
        print(f"WARNING: Not enough values! Missing {expected_count - len(values)}")

    return values[:expected_count]

def generate_rust_code(values, output_dir):
    """Generate Rust code from the wavetable values."""

    os.makedirs(output_dir, exist_ok=True)

    # Generate the main wavetable data file
    main_file = os.path.join(output_dir, "mod.rs")

    print(f"Generating {main_file}...")

    with open(main_file, 'w') as f:
        f.write("//! Wavetable data for Odin 2 synthesizer\n")
        f.write("//!\n")
        f.write("//! Auto-generated from WavetableData.cpp\n")
        f.write("//! DO NOT EDIT MANUALLY\n\n")

        f.write("use crate::constants::{NUMBER_OF_WAVETABLES, SUBTABLES_PER_WAVETABLE, WAVETABLE_LENGTH};\n\n")

        # Wavetable names (basic ones for now)
        f.write("/// Wavetable names\n")
        f.write("pub const WAVETABLE_NAMES: [&str; 160] = [\n")

        basic_names = [
            "Sine", "FatSaw", "Triangle", "Square",  # 0-3: Basic waves
        ]

        for i in range(NUMBER_OF_WAVETABLES):
            if i < len(basic_names):
                f.write(f'    "{basic_names[i]}",\n')
            else:
                f.write(f'    "Wavetable{i}",\n')

        f.write("];\n\n")

        # Include the data modules
        for i in range(NUMBER_OF_WAVETABLES):
            f.write(f"mod wt_{i:03d};\n")

        f.write("\n")

        # Create the getter function
        f.write("/// Get a wavetable by index\n")
        f.write("/// Returns a reference to the subtables array [33][512]\n")
        f.write("pub fn get_wavetable(index: usize) -> &'static [[f32; WAVETABLE_LENGTH]; SUBTABLES_PER_WAVETABLE] {\n")
        f.write("    match index {\n")

        for i in range(NUMBER_OF_WAVETABLES):
            f.write(f"        {i} => &wt_{i:03d}::DATA,\n")

        f.write("        _ => &wt_000::DATA, // Default to first wavetable\n")
        f.write("    }\n")
        f.write("}\n\n")

        # Get subtable function
        f.write("/// Get a specific subtable from a wavetable\n")
        f.write("pub fn get_subtable(wavetable: usize, subtable: usize) -> &'static [f32; WAVETABLE_LENGTH] {\n")
        f.write("    &get_wavetable(wavetable)[subtable.min(SUBTABLES_PER_WAVETABLE - 1)]\n")
        f.write("}\n")

    # Generate individual wavetable files
    idx = 0
    for wt in range(NUMBER_OF_WAVETABLES):
        wt_file = os.path.join(output_dir, f"wt_{wt:03d}.rs")

        if wt % 20 == 0:
            print(f"Generating wavetable {wt}/{NUMBER_OF_WAVETABLES}...")

        with open(wt_file, 'w') as f:
            f.write(f"//! Wavetable {wt} data\n")
            f.write("//! Auto-generated - DO NOT EDIT\n\n")
            f.write("#![allow(clippy::excessive_precision)]\n\n")
            f.write("use crate::constants::{SUBTABLES_PER_WAVETABLE, WAVETABLE_LENGTH};\n\n")
            f.write("pub const DATA: [[f32; WAVETABLE_LENGTH]; SUBTABLES_PER_WAVETABLE] = [\n")

            for st in range(SUBTABLES_PER_WAVETABLE):
                f.write("    [\n        ")

                for s in range(WAVETABLE_LENGTH):
                    val = values[idx] if idx < len(values) else 0.0
                    idx += 1

                    # Format the float
                    if val == 0.0:
                        f.write("0.0")
                    elif abs(val) < 1e-10:
                        f.write("0.0")  # Treat very small values as 0
                    else:
                        f.write(f"{val}")

                    if s < WAVETABLE_LENGTH - 1:
                        f.write(", ")
                        if (s + 1) % 8 == 0:
                            f.write("\n        ")

                f.write("\n    ],\n")

            f.write("];\n")

    print(f"Generated {NUMBER_OF_WAVETABLES} wavetable files")
    return True

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)

    cpp_file = os.path.join(
        project_root,
        "odin2/Source/audio/Oscillators/Wavetables/Tables/WavetableData.cpp"
    )

    output_dir = os.path.join(
        project_root,
        "crates/odin2-core/src/wavetables"
    )

    if not os.path.exists(cpp_file):
        print(f"ERROR: C++ file not found: {cpp_file}")
        sys.exit(1)

    values = parse_wavetable_cpp(cpp_file)

    if values is None:
        print("ERROR: Failed to parse wavetable data")
        sys.exit(1)

    if generate_rust_code(values, output_dir):
        print(f"\nSuccess! Generated Rust wavetables in {output_dir}")
        print(f"Total data size: ~{len(values) * 4 / 1024 / 1024:.1f} MB")
    else:
        print("ERROR: Failed to generate Rust code")
        sys.exit(1)

if __name__ == "__main__":
    main()
