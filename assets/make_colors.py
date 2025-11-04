from itertools import chain
from contextlib import redirect_stdout
import re

def main():
    tmp = {}
    for ident, rgb in chain(parse_svg(), parse_x11()):
        id = ''.join(ident)
        rust_name = '_'.join(part.upper() for part in ident)
        display_name = ' '.join(ident)

        tmp[id] = (rust_name, display_name, rgb)

    with open('../src/colors.rs', 'wt') as f:
        with redirect_stdout(f):
            generate_file(tmp)
    

def generate_file(color_dict):
    colors = list(color_dict.values())
    colors.sort(key=lambda v: make_key(v[0]))

    print("use egui::Color32;")
    print()
    for (rust_name, _, (r, g, b)) in colors:
        print(f"pub const {rust_name}: Color32 = Color32::from_rgb({r}, {g}, {b});")

    print()
    n = len(colors)
    print(f"pub const ALL: [(Color32, &str); {n}] = [")
    for (rust_name, display_name, _) in colors:
        print(f'({rust_name}, "{display_name}"),')

    print("];")


def make_key(s):
    m = re.match("([A-Z_]+)([0-9]*)", s)
    base, num = m.groups()

    if num != '':
        num = int(num)
    else:
        num = -1

    return (base, num)


def parse_x11():
    for line in open('rgb.txt'):
        if line.startswith('!'):
            continue

        fields = line.split()
        name = fields[3]
        if name[0].isupper() or 'grey' in line:
            continue

        r = int(fields[0])
        g = int(fields[1])
        b = int(fields[2])
        name = fields[3:]

        yield (name, (r, g, b))


def parse_svg():
    for line in open('svg.txt'):
        if line.startswith('!'):
            continue

        fields = line.split()
        name = fields[0]
        r = int(fields[1])
        g = int(fields[2])
        b = int(fields[3])
        
        yield ([name], (r, g, b))


if __name__ == '__main__':
    main()
