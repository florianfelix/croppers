#!/usr/bin/env python3
"""
Rasterizes TTF fonts into 1-bit bitmaps and formats them as WGSL hex constants
for 8x8, 8x16, 16x16, and 16x32 resolutions.
"""

import sys
import os
from PIL import Image, ImageDraw, ImageFont

CHARS = [' ', '.', ':', ';', '-', '=', '+', 'x', 'X', '8', '0', 'S', '*', '%', '#', '@']

def render_glyph(font, ch, width, height, threshold=100):
    if ch == ' ':
        return Image.new('1', (width, height), 0)

    # Render at higher resolution with anti-aliasing for cleaner thresholding
    supersample = 4
    sw, sh = width * supersample, height * supersample

    # Find size that fits bounding box nicely with a slight margin
    font_size = int(sh * 0.95)
    f = font.font_variant(size=font_size)

    # Render on grayscale canvas
    img = Image.new('L', (sw, sh), 0)
    draw = ImageDraw.Draw(img)

    bbox = draw.textbbox((0, 0), ch, font=f)
    bw = bbox[2] - bbox[0]
    bh = bbox[3] - bbox[1]

    # Center glyph in canvas
    x = (sw - bw) // 2 - bbox[0]
    y = (sh - bh) // 2 - bbox[1]
    draw.text((x, y), ch, font=f, fill=255)

    # Downsample back to target width/height with box filter
    img_down = img.resize((width, height), Image.Resampling.BOX)

    # Threshold to binary 1-bit
    return img_down.point(lambda p: 1 if p > threshold else 0, mode='1')

def pack_8x8(img):
    # 8 rows x 8 cols -> 2 x u32
    # word 0: rows 0..3 (bits 31..0, row r has bits 31 - (r*8 + c))
    # word 1: rows 4..7
    words = []
    for chunk in range(2):
        w = 0
        for r in range(4):
            row_idx = chunk * 4 + r
            for c in range(8):
                if img.getpixel((c, row_idx)):
                    bit = 31 - (r * 8 + c)
                    w |= (1 << bit)
        words.append(w)
    return f"vec2<u32>(0x{words[0]:08x}u, 0x{words[1]:08x}u)"

def pack_8x16(img):
    # 16 rows x 8 cols -> 4 x u32 (vec4<u32>)
    words = []
    for chunk in range(4):
        w = 0
        for r in range(4):
            row_idx = chunk * 4 + r
            for c in range(8):
                if img.getpixel((c, row_idx)):
                    bit = 31 - (r * 8 + c)
                    w |= (1 << bit)
        words.append(w)
    return f"vec4<u32>(0x{words[0]:08x}u, 0x{words[1]:08x}u, 0x{words[2]:08x}u, 0x{words[3]:08x}u)"

def pack_16x16(img):
    # 16 rows x 16 cols -> 8 x u32 (each u32 holds 2 rows: upper 16 bits = even row, lower 16 bits = odd row)
    words = []
    for r_pair in range(8):
        w = 0
        # even row (bits 31..16)
        r0 = r_pair * 2
        for c in range(16):
            if img.getpixel((c, r0)):
                w |= (1 << (31 - c))
        # odd row (bits 15..0)
        r1 = r_pair * 2 + 1
        for c in range(16):
            if img.getpixel((c, r1)):
                w |= (1 << (15 - c))
        words.append(f"0x{w:08x}u")
    return words

def pack_16x32(img):
    # 32 rows x 16 cols -> 16 x u32 (each u32 holds 2 rows)
    words = []
    for r_pair in range(16):
        w = 0
        r0 = r_pair * 2
        for c in range(16):
            if img.getpixel((c, r0)):
                w |= (1 << (31 - c))
        r1 = r_pair * 2 + 1
        for c in range(16):
            if img.getpixel((c, r1)):
                w |= (1 << (15 - c))
        words.append(f"0x{w:08x}u")
    return words

def main():
    font_path = sys.argv[1] if len(sys.argv) > 1 else "/usr/share/fonts/TTF/DejaVuSansMono-Bold.ttf"
    if not os.path.isfile(font_path):
        print(f"Error: font file '{font_path}' not found.", file=sys.stderr)
        sys.exit(1)

    font = ImageFont.truetype(font_path, 32)
    print(f"// Generated from: {font_path}\n")

    # 1. 8x8
    print("// --- Option 0: 8x8 Glyphs (vec2<u32>) ---")
    print("const GLYPHS_8X8 = array<vec2<u32>, 16>(")
    for i, ch in enumerate(CHARS):
        img = render_glyph(font, ch, 8, 8)
        packed = pack_8x8(img)
        comma = "," if i < 15 else ""
        print(f"    {packed}{comma} // {i}: '{ch}'")
    print(");\n")

    # 2. 8x16
    print("// --- Option 1: 8x16 Glyphs (vec4<u32>) ---")
    print("const GLYPHS_8X16 = array<vec4<u32>, 16>(")
    for i, ch in enumerate(CHARS):
        img = render_glyph(font, ch, 8, 16)
        packed = pack_8x16(img)
        comma = "," if i < 15 else ""
        print(f"    {packed}{comma} // {i}: '{ch}'")
    print(");\n")

    # 3. 16x16
    print("// --- Option 2: 16x16 Glyphs (array<u32, 128>) ---")
    print("const GLYPHS_16X16 = array<u32, 128>(")
    for i, ch in enumerate(CHARS):
        img = render_glyph(font, ch, 16, 16)
        words = pack_16x16(img)
        print(f"    // Glyph {i}: '{ch}'")
        print("    " + ", ".join(words[:4]) + ",")
        comma = "," if i < 15 else ""
        print("    " + ", ".join(words[4:]) + f"{comma}")
    print(");\n")

    # 4. 16x32
    print("// --- Option 3: 16x32 Glyphs (array<u32, 256>) ---")
    print("const GLYPHS_16X32 = array<u32, 256>(")
    for i, ch in enumerate(CHARS):
        img = render_glyph(font, ch, 16, 32)
        words = pack_16x32(img)
        print(f"    // Glyph {i}: '{ch}'")
        for chunk in range(4):
            part = words[chunk*4 : (chunk+1)*4]
            comma = "," if (i < 15 or chunk < 3) else ""
            print("    " + ", ".join(part) + comma)
    print(");")

if __name__ == '__main__':
    main()
