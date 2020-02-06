#!/usr/bin/env python

import os
import sys


IGNORE_TESTS = [
    'macos.tests',
]

IGNORE_TEST_CASES = [
    # aots tests
    # Unknown issue. Investigate.
    'gpos_context2_classes_001',
    'gpos_context2_classes_002',

    # in-house tests
    # Unknown issue. Investigate.
    'indic_joiners_003',
    'indic_joiners_004',
    'indic_joiners_005',
    'indic_joiners_006',
    'simple_002',
    'use_010',
    'vertical_001',
    'vertical_002',
    # dfont fonts are not supported
    'collections_001',
    'collections_002',
    'collections_003',
    # Arabic fallback shaping requires
    'arabic_fallback_shaping_001',

    # text-rendering-tests tests
    # Incorrect CMAP 14 parsing. Was fixed in master, but broken in 2.6.4
    'cmap_1_001',
    'cmap_1_002',
    'cmap_1_003',
    'cmap_1_004',
    'cmap_2_001',
    'cmap_2_002',
    # Unknown issue. Investigate.
    'gsub_3_001',
    'gvar_2_001',
    'gvar_2_005',
    'morx_14_002',
    'morx_24_001',
    'morx_29_001',
    'morx_29_002',
    'morx_29_003',
    'morx_29_004',
    'morx_30_001',
    'morx_30_002',
    'morx_30_003',
    'morx_30_004',
    'morx_34_001',
    'morx_36_001',
    'morx_41_003',
    'morx_41_004',
    'morx_6_001',
    # `true` fonts are not supported
    'gvar_4_001',
    'gvar_4_002',
    'gvar_4_003',
    'gvar_4_004',
    'gvar_4_005',
    'gvar_4_006',
    'gvar_4_007',
    'gvar_4_008',
    'gvar_4_009',
    'gvar_4_010',
    'gvar_4_011',
    'gvar_5_001',
    'gvar_5_002',
    'gvar_5_003',
    'gvar_5_004',
    'gvar_5_005',
    'gvar_5_006',
    'gvar_5_007',
    'gvar_5_008',
    'gvar_5_009',
    'gvar_5_010',
    'gvar_5_011',
    'gvar_6_001',
    'gvar_6_002',
    'gvar_6_003',
    'gvar_6_004',
    'gvar_6_005',
    'gvar_6_006',
    'gvar_6_007',
    'gvar_6_008',
    'gvar_6_009',
    'gvar_6_010',
    'gvar_6_011',
]

TEST_OVERRIDES = {
    # FreeType has a bit different rounding
    'shknda_1_001': 'knLI|knLAc2@756,0',
    'shknda_1_008': 'knPHI|knRAc2@734,0',
    'shknda_1_012': 'knYI|knAnusvara@1251,0',
    'shknda_1_015': 'knGI|knLAc2@620,0',
    'shknda_1_019': 'knTI|knLengthmark@612,0',
    'shknda_1_027': 'knLI|knLAc2@756,0',
    'shknda_1_030': 'knVI|knAnusvara@748,0',
    'gsub_2_008': 'uni1373.init|uni136B.medi@621,0|uni137B.fina@1101,0',
    'gsub_2_009': 'uni1373.init|uni136B.medi@621,0|uni137B.medi@1101,0|uni1373.medi@1488,0|uni136B.fina@2109,0',
    'gsub_2_010': 'uni1373.init|uni136B.medi@621,0|uni137B.medi@1101,0|uni1375.medi@1488,0|uni136D.fina@2155,0',
    # Failed to resolve glyph names
    'kern_1_001': 'gid2|gid1|gid3@400,0|gid1@600,0|gid3@1000,0|gid1@1200,0|gid2@1600,0',
    'kern_2_001': 'gid3|gid2@400,0|gid2@1100,0|gid1@1100,0|gid2@1500,0|gid2@2200,0|gid1@2200,0|gid2@2600,'
                  '0|gid2@3300,0|gid3@3500,0',
    'gpos_2_001': 'gid1',
    'gpos_2_002': 'gid2',
    'gvar_2_001': 'gid2|gid3@500,0|gid1@1000,0',
    'gvar_2_005': 'gid2|gid3@500,0|gid1@1000,0',
    'gsub_1_001': 'gid2|gid3@500,0|gid1@1000,0',
    'gpos_2_003': 'gid1|gid2',
    # Different error processing
    'collections_006': 'malformed font',
    'indic_decompose_001': 'malformed font',
}


def update_relative_path(dir, path):
    name = os.path.basename(os.path.dirname(dir))
    path = path.replace('..', name)
    return path


# Converts `U+0041,U+0078` into `\u{0041}\u{0078}`
def parse_unicodes(unicodes):
    text = ''
    for (i, u) in enumerate(unicodes.split(',')):
        if i > 0 and i % 10 == 0:
            text += '\\\n             '

        text += f'\\u{{{u[2:]}}}'

    return text


def convert_test(dir, file_name, idx, data, fonts):
    fontfile, options, unicodes, glyphs_expected = data.split(':')

    fontfile = update_relative_path(dir, fontfile)

    unicodes = parse_unicodes(unicodes)

    test_name = file_name.replace('.tests', '').replace('-', '_') + f'_{idx:03d}'
    test_name = test_name.lower()

    if test_name in TEST_OVERRIDES:
        glyphs_expected = TEST_OVERRIDES[test_name]
    else:
        glyphs_expected = glyphs_expected[1:-1]  # remove `[..]`
        glyphs_expected = glyphs_expected.replace('|', '|\\\n         ')

    options = options.replace('"', '\\"')

    fonts.add(os.path.split(fontfile)[1])

    if test_name in IGNORE_TEST_CASES:
        return

    print(f'#[test]')
    print(f'fn {test_name}() {{')
    print(f'    assert_eq!(')
    print(f'        shape(')
    print(f'            "{fontfile}",')
    print(f'            "{unicodes}",')
    print(f'            "{options}",')
    print(f'        ),')
    print(f'        "{glyphs_expected}"')
    print(f'    );')
    print(f'}}')
    print()


def convert(dir):
    files = sorted(os.listdir(dir))
    files = [f for f in files if f.endswith('.tests')]

    fonts = set()

    for file in files:
        if file in IGNORE_TESTS:
            continue

        with open(dir + '/' + file, 'r') as f:
            for idx, test in enumerate(f.read().splitlines()):
                # skip comments and empty lines
                if test.startswith('#') or len(test) == 0:
                    continue

                convert_test(dir, file, idx + 1, test, fonts)

    return fonts


if len(sys.argv) != 2:
    print('Usage:   gen-shaping-tests.py /path/to/harfbuzz-src/test/shaping/data/*/tests')
    print('Example: gen-shaping-tests.py ~/harfbuzz-2.6.4/test/shaping/data/in-house/tests > '
          '../tests/shaping_in_house.rs')
    exit(1)

print('// WARNING: this file was generated by ../scripts/gen-shaping-tests.py')
print()
print('use pretty_assertions::assert_eq;')
print()
print('mod shaping_impl;')
print('use shaping_impl::shape;')
print()

used_fonts = convert(sys.argv[1])

# check for unused fonts
tests_name = os.path.basename(os.path.dirname(sys.argv[1]))
font_files = set(os.listdir(f'../harfbuzz/test/shaping/data/{tests_name}/fonts/'))
unused_fonts = sorted(list(font_files.difference(used_fonts)))
if len(unused_fonts) != 0:
    print('Unused fonts:', file=sys.stderr)
    for font in unused_fonts:
        print(font, file=sys.stderr)
