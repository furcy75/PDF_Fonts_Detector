"""Génère assets/app_icon.png (256x256) et assets/app_icon.ico (multi-résolutions) sans dépendance externe."""

from __future__ import annotations

import math
import struct
import zlib
from pathlib import Path


def clamp(v: float, lo: float = 0.0, hi: float = 1.0) -> float:
    return lo if v < lo else (hi if v > hi else v)


def mix(a: float, b: float, t: float) -> float:
    return a + (b - a) * t


def mix_rgb(c1: tuple[float, float, float], c2: tuple[float, float, float], t: float) -> tuple[float, float, float]:
    t = clamp(t)
    return (mix(c1[0], c2[0], t), mix(c1[1], c2[1], t), mix(c1[2], c2[2], t))


def sd_rounded_rect(px: float, py: float, cx: float, cy: float, hx: float, hy: float, r: float) -> float:
    dx = abs(px - cx) - (hx - r)
    dy = abs(py - cy) - (hy - r)
    outside = math.hypot(max(dx, 0.0), max(dy, 0.0))
    inside = min(max(dx, dy), 0.0)
    return outside + inside - r


def sd_segment(px: float, py: float, ax: float, ay: float, bx: float, by: float) -> float:
    pax, pay = px - ax, py - ay
    bax, bay = bx - ax, by - ay
    denom = bax * bax + bay * bay
    h = clamp((pax * bax + pay * bay) / denom) if denom > 1e-9 else 0.0
    return math.hypot(pax - bax * h, pay - bay * h)


def blend(dst: tuple[float, float, float, float], src: tuple[float, float, float, float]) -> tuple[float, float, float, float]:
    sr, sg, sb, sa = src
    dr, dg, db, da = dst
    out_a = sa + da * (1.0 - sa)
    if out_a < 1e-6:
        return (0.0, 0.0, 0.0, 0.0)
    out_r = (sr * sa + dr * da * (1.0 - sa)) / out_a
    out_g = (sg * sa + dg * da * (1.0 - sa)) / out_a
    out_b = (sb * sa + db * da * (1.0 - sa)) / out_a
    return (out_r, out_g, out_b, out_a)


def sample_icon(x: float, y: float) -> tuple[float, float, float, float]:
    """Échantillonne l'icône en coordonnées normalisées 0..256."""
    col = (0.0, 0.0, 0.0, 0.0)
    t_diag = clamp((x + y) / 512.0)

    # 1. Fond squircle ardoise + bordure bleu->indigo
    d_outer = sd_rounded_rect(x, y, 128.0, 128.0, 116.0, 116.0, 52.0)
    if d_outer < 1.5:
        aa = clamp(0.5 - d_outer)
        bg_r, bg_g, bg_b = mix_rgb((30, 41, 59), (15, 23, 42), t_diag)
        col = blend(col, (bg_r, bg_g, bg_b, aa))

        # Bordure lumineuse (largeur 7px)
        d_border = abs(d_outer + 3.5) - 3.5
        if d_border < 1.5:
            ba = clamp(0.5 - d_border) * aa
            br, bg, bb = mix_rgb((56, 189, 248), (99, 102, 241), t_diag)
            col = blend(col, (br, bg, bb, ba))

    # 2. Feuille document PDF (gauche/haut)
    d_doc = sd_rounded_rect(x, y, 113.0, 128.0, 73.0, 84.0, 16.0)
    # Coin plié en haut à droite du document : droite passant par (146, 44) et (186, 84) => x - y = 102
    d_cut = (x - y - 102.0) * 0.70710678
    d_doc_shape = max(d_doc, d_cut)
    if d_doc_shape < 1.5:
        aa = clamp(0.5 - d_doc_shape)
        col = blend(col, (30.0, 41.0, 59.0, aa))
        # Contour du document
        d_doc_edge = abs(d_doc_shape + 2.5) - 2.5
        if d_doc_edge < 1.5:
            col = blend(col, (100.0, 116.0, 139.0, clamp(0.5 - d_doc_edge) * aa))

        # Triangle coin plié en haut à droite
        if x >= 146.0 and y <= 84.0 and d_cut <= 0.5:
            col = blend(col, (71.0, 85.0, 105.0, aa))

    # 3. Pastille rouge "PDF"
    d_pdf = sd_rounded_rect(x, y, 93.0, 77.0, 39.0, 15.0, 8.0)
    if d_pdf < 1.5:
        aa = clamp(0.5 - d_pdf)
        pr, pg, pb = mix_rgb((244, 63, 94), (225, 29, 72), (x - 54.0) / 78.0)
        col = blend(col, (pr, pg, pb, aa))

        # Mini glyphes "PDF" vectoriels nets
        d_txt = min(
            # P
            sd_segment(x, y, 67, 70, 67, 84),
            sd_segment(x, y, 67, 70, 75, 70),
            sd_segment(x, y, 75, 70, 75, 77),
            sd_segment(x, y, 67, 77, 75, 77),
            # D
            sd_segment(x, y, 86, 70, 86, 84),
            sd_segment(x, y, 86, 70, 93, 70),
            sd_segment(x, y, 93, 70, 96, 73),
            sd_segment(x, y, 96, 73, 96, 81),
            sd_segment(x, y, 96, 81, 93, 84),
            sd_segment(x, y, 86, 84, 93, 84),
            # F
            sd_segment(x, y, 107, 70, 107, 84),
            sd_segment(x, y, 107, 70, 116, 70),
            sd_segment(x, y, 107, 77, 114, 77),
        ) - 1.75
        if d_txt < 1.5:
            col = blend(col, (255.0, 255.0, 255.0, clamp(0.5 - d_txt)))

    # 4. Lignes de texte stylisées sur le document
    for ly, lw in ((112.0, 47.0), (130.0, 35.0), (148.0, 27.0)):
        d_line = sd_rounded_rect(x, y, 56.0 + lw, ly, lw, 4.0, 4.0)
        if d_line < 1.5:
            c = (71.0, 85.0, 105.0) if ly < 120.0 else (51.0, 65.0, 85.0)
            col = blend(col, (c[0], c[1], c[2], clamp(0.5 - d_line)))

    # 5. Badge circulaire bleu "Aa" en bas à droite
    dist_badge = math.hypot(x - 176.0, y - 176.0)
    # Anneau de découpe sombre (rayon 58)
    d_halo = dist_badge - 58.0
    if d_halo < 1.5:
        col = blend(col, (15.0, 23.0, 42.0, clamp(0.5 - d_halo)))

    # Disque bleu dégradé (rayon 51)
    d_disc = dist_badge - 51.0
    if d_disc < 1.5:
        aa = clamp(0.5 - d_disc)
        t_b = clamp((x - 125.0 + y - 125.0) / 204.0)
        br, bg, bb = mix_rgb((56, 189, 248), (2, 132, 199), t_b)
        col = blend(col, (br, bg, bb, aa))

        # Grand "A" vectoriel net (gauche du disque)
        d_big_a = min(
            sd_segment(x, y, 145, 196, 160, 152),
            sd_segment(x, y, 160, 152, 175, 196),
            sd_segment(x, y, 151, 182, 169, 182),
        ) - 4.0

        # Minuscule "a" vectorielle nette (droite du disque)
        # Panse circulaire + tige droite + chapeau supérieur
        d_a_bowl = abs(math.hypot(x - 194.5, y - 186.5) - 9.5) - 3.3
        d_a_stem = sd_segment(x, y, 204.0, 171.0, 204.0, 196.0) - 3.5
        d_a_top = sd_segment(x, y, 187.0, 169.0, 201.0, 169.0) - 3.3
        d_a_curve = sd_segment(x, y, 201.0, 169.0, 204.0, 173.0) - 3.3
        d_small_a = min(d_a_bowl, d_a_stem, d_a_top, d_a_curve)

        d_aa = min(d_big_a, d_small_a)
        if d_aa < 1.5:
            col = blend(col, (255.0, 255.0, 255.0, clamp(0.5 - d_aa)))

    return col


def render_rgba(size: int, samples: int = 3) -> bytes:
    """Rend l'icône en RGBA8888 avec suréchantillonnage NxN anti-crénelage."""
    out = bytearray(size * size * 4)
    scale = 256.0 / size
    inv_s2 = 1.0 / (samples * samples)

    for py in range(size):
        for px in range(size):
            r = g = b = a = 0.0
            for sy in range(samples):
                yy = (py + (sy + 0.5) / samples) * scale
                for sx in range(samples):
                    xx = (px + (sx + 0.5) / samples) * scale
                    sr, sg, sb, sa = sample_icon(xx, yy)
                    r += sr * sa
                    g += sg * sa
                    b += sb * sa
                    a += sa
            a *= inv_s2
            if a > 1e-6:
                r = (r * inv_s2) / a
                g = (g * inv_s2) / a
                b = (b * inv_s2) / a
            else:
                r = g = b = 0.0
            idx = (py * size + px) * 4
            out[idx] = int(clamp(r, 0.0, 255.0) + 0.5)
            out[idx + 1] = int(clamp(g, 0.0, 255.0) + 0.5)
            out[idx + 2] = int(clamp(b, 0.0, 255.0) + 0.5)
            out[idx + 3] = int(clamp(a * 255.0, 0.0, 255.0) + 0.5)
    return bytes(out)


def encode_png(width: int, height: int, rgba: bytes) -> bytes:
    def chunk(tag: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    raw_rows = bytearray()
    stride = width * 4
    for y in range(height):
        raw_rows.append(0)  # Filter type 0 (None)
        raw_rows.extend(rgba[y * stride : (y + 1) * stride])

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    idat = zlib.compress(bytes(raw_rows), level=9)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b"")


def encode_ico(png_entries: list[tuple[int, bytes]]) -> bytes:
    """Crée un fichier .ico moderne (entrées PNG embarquées multi-tailles, supporté depuis Windows Vista+)."""
    count = len(png_entries)
    header = struct.pack("<HHH", 0, 1, count)
    offset = 6 + 16 * count
    dir_entries = bytearray()
    payload = bytearray()

    for size, png_data in png_entries:
        dim_byte = 0 if size >= 256 else size
        dir_entries.extend(
            struct.pack("<BBBBHHII", dim_byte, dim_byte, 0, 0, 1, 32, len(png_data), offset)
        )
        payload.extend(png_data)
        offset += len(png_data)

    return header + bytes(dir_entries) + bytes(payload)


def main() -> None:
    assets_dir = Path(__file__).resolve().parent
    sizes = [256, 128, 64, 48, 32, 16]
    png_entries: list[tuple[int, bytes]] = []

    for sz in sizes:
        samples = 2 if sz >= 128 else 4
        rgba = render_rgba(sz, samples=samples)
        png_bytes = encode_png(sz, sz, rgba)
        png_entries.append((sz, png_bytes))
        if sz == 256:
            (assets_dir / "app_icon.png").write_bytes(png_bytes)

    ico_bytes = encode_ico(png_entries)
    (assets_dir / "app_icon.ico").write_bytes(ico_bytes)
    print("Generated assets/app_icon.png and assets/app_icon.ico successfully.")


if __name__ == "__main__":
    main()
