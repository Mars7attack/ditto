#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["numpy==2.5.3", "pillow==12.3.0"]
# ///
"""Measure braille dot positions, independent of screenshot zoom/font rasterization.

Usage: uv run scripts/compare-braille-captures.py A.png B.png --crop-b x,y,w,h --out DIR
Crop to the artwork/preview (not its UI). No input image is modified.
Writes dot masks, reconstructed dot occupancy, an aligned comparison and JSON metrics.
"""
import argparse
import json
from collections import deque
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw


def crop_arg(value):
    x, y, w, h = map(int, value.split(","))
    return x, y, x + w, y + h


def clusters(values, tolerance=0.8):
    groups = []
    for value in sorted(values):
        if not groups or value - np.mean(groups[-1]) > tolerance:
            groups.append([value])
        else:
            groups[-1].append(value)
    return np.array([np.mean(group) for group in groups])


def extract(path, crop, threshold):
    im = Image.open(path).convert("RGB")
    if crop:
        im = im.crop(crop)
    rgb = np.asarray(im).astype(float)
    background = np.median(rgb.reshape(-1, 3), axis=0)
    strength = np.max(np.abs(rgb - background), axis=2)
    peak = np.percentile(strength[strength > 10], 99)
    mask = strength > threshold * peak
    seen = np.zeros(mask.shape, dtype=bool)
    dots = []
    for y, x in zip(*np.nonzero(mask)):
        if seen[y, x]:
            continue
        pending = deque([(int(x), int(y))])
        seen[y, x] = True
        pixels = []
        while pending:
            xx, yy = pending.popleft()
            pixels.append((xx, yy))
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    nx, ny = xx + dx, yy + dy
                    if (0 <= nx < mask.shape[1] and 0 <= ny < mask.shape[0]
                            and mask[ny, nx] and not seen[ny, nx]):
                        seen[ny, nx] = True
                        pending.append((nx, ny))
        p = np.array(pixels)
        extent = p.max(axis=0) - p.min(axis=0) + 1
        # Reject preview borders or unrelated connected UI. Keep tiny antialiased dots.
        if len(p) > 100 or max(extent) > 12:
            continue
        weights = strength[p[:, 1], p[:, 0]]
        center = np.average(p + 0.5, axis=0, weights=weights)
        dots.append(center)
    dots = np.array(dots)
    if len(dots) == 0:
        raise ValueError(f"No dots found: {path}")
    xs, ys = clusters(dots[:, 0]), clusters(dots[:, 1])
    occupancy = np.zeros((len(ys), len(xs)), dtype=bool)
    for x, y in dots:
        occupancy[np.argmin(abs(ys - y)), np.argmin(abs(xs - x))] = True
    return im, mask, dots, xs, ys, occupancy


def metrics(data):
    _, _, dots, xs, ys, occupancy = data
    def spacing(axis):
        gaps = np.diff(axis)
        rounded, counts = np.unique(np.round(gaps, 1), return_counts=True)
        return {"median": float(np.median(gaps)), "min": float(gaps.min()),
                "max": float(gaps.max()), "gap_histogram": dict(zip(map(str, rounded), map(int, counts)))}
    return {"dots": len(dots), "columns": len(xs), "rows": len(ys),
            "x_spacing": spacing(xs), "y_spacing": spacing(ys),
            "occupied": int(occupancy.sum()), "x_centres": xs.tolist(), "y_centres": ys.tolist()}


def fit_text_metrics(external, reference):
    """Fit space advance, braille advance and dot offsets without resizing images.

    Reference B must have a uniform dot grid. Pair dots by row and left-to-right
    order only when each row has the same count. Search all 2x4 cell phases.
    Low residual is evidence for a font-metrics deformation, not missing dots.
    """
    _, _, adots, _, ay, _ = external
    _, _, bdots, bx, by, _ = reference
    if any(np.std(np.diff(axis)) / np.mean(np.diff(axis)) > 0.1 for axis in (bx, by)):
        return {"comparable": False, "reason": "Reference B must have a uniformly spaced dot grid"}
    if len(ay) != len(by):
        return {"comparable": False, "reason": "Different detected row counts"}
    paired = []
    for row in range(len(by)):
        a = sorted((p for p in adots if np.argmin(abs(ay - p[1])) == row), key=lambda p: p[0])
        b = sorted((p for p in bdots if np.argmin(abs(by - p[1])) == row), key=lambda p: p[0])
        if len(a) != len(b):
            return {"comparable": False, "reason": f"Row {row} has {len(a)} vs {len(b)} dots"}
        paired.extend((int(np.argmin(abs(bx - bp[0]))), row, ap[0], ap[1]) for ap, bp in zip(a, b))
    paired = np.array(paired)
    candidates = []
    for phase_x in range(2):
        for phase_y in range(4):
            cx = ((paired[:, 0] + phase_x) // 2).astype(int)
            cy = ((paired[:, 1] + phase_y) // 4).astype(int)
            ink_cells = set(zip(cx, cy))
            preceding = np.array([sum((col, row) in ink_cells for col in range(x)) for x, row in zip(cx, cy)])
            xmodel = np.column_stack([np.ones(len(cx)), cx, preceding, (paired[:, 0] + phase_x) % 2])
            ymodel = np.column_stack([np.ones(len(cy)), cy, (paired[:, 1] + phase_y) % 4])
            xp = np.linalg.lstsq(xmodel, paired[:, 2], rcond=None)[0]
            yp = np.linalg.lstsq(ymodel, paired[:, 3], rcond=None)[0]
            residual = np.column_stack([xmodel @ xp - paired[:, 2], ymodel @ yp - paired[:, 3]])
            candidates.append({"cell_phase": [phase_x, phase_y], "paired_dots": len(cx),
                "space_advance_px": float(xp[1]), "braille_advance_px": float(xp[1] + xp[2]),
                "braille_minus_space_px": float(xp[2]), "dot_column_gap_px": float(xp[3]),
                "line_advance_px": float(yp[1]), "dot_row_gap_px": float(yp[2]),
                "rms_position_error_px": float(np.sqrt(np.mean(residual ** 2))),
                "max_position_error_px": float(abs(residual).max())})
    best = min(candidates, key=lambda candidate: candidate["rms_position_error_px"])
    best["model_supported"] = best["rms_position_error_px"] < 0.35 and best["max_position_error_px"] < 1
    return best


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("a", type=Path)
    parser.add_argument("b", type=Path)
    parser.add_argument("--crop-a", type=crop_arg)
    parser.add_argument("--crop-b", type=crop_arg)
    parser.add_argument("--threshold", type=float, default=0.35)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    datasets = [extract(args.a, args.crop_a, args.threshold), extract(args.b, args.crop_b, args.threshold)]
    report = {"threshold_fraction": args.threshold, "a": metrics(datasets[0]), "b": metrics(datasets[1])}
    report["font_metrics_fit"] = fit_text_metrics(datasets[0], datasets[1])
    fitted = report["font_metrics_fit"]
    if fitted.get("model_supported"):
        # Diagnostic reconstruction, not a claim to recover the original file:
        # screenshots cannot distinguish U+0020 from an invisible U+2800.
        px, py = fitted["cell_phase"]
        occupancy = datasets[1][-1]
        cells = np.zeros(((occupancy.shape[0] + py + 3) // 4,
                          (occupancy.shape[1] + px + 1) // 2), dtype=int)
        bits = ((0, 1, 2, 6), (3, 4, 5, 7))
        for y, x in zip(*np.nonzero(occupancy)):
            xx, yy = x + px, y + py
            cells[yy // 4, xx // 2] |= 1 << bits[xx % 2][yy % 4]
        text = "\n".join("".join(chr(0x2800 + v) if v else " " for v in row) for row in cells)
        (args.out / "reconstructed-reference.txt").write_text(text)
        (args.out / "reconstructed-braille-spaces.txt").write_text(text.replace(" ", "\u2800"))
    for label, (im, mask, dots, xs, ys, occupancy) in zip("ab", datasets):
        im.save(args.out / f"{label}-crop.png")
        Image.fromarray((mask * 255).astype("uint8")).save(args.out / f"{label}-mask.png")
        # Compare topology after removing the actual font's nonuniform spacing.
        grid = Image.new("RGB", (len(xs) * 8, len(ys) * 8), "#111111")
        draw = ImageDraw.Draw(grid)
        for y, x in zip(*np.nonzero(occupancy)):
            draw.ellipse((x * 8 + 2, y * 8 + 2, x * 8 + 5, y * 8 + 5), fill="white")
        grid.save(args.out / f"{label}-uniform-lattice.png")
    a, b = datasets[0][-1], datasets[1][-1]
    if a.shape == b.shape:
        report["topology"] = {"matching": int((a == b).sum()), "different": int((a != b).sum()),
                              "intersection_over_union": float((a & b).sum() / (a | b).sum())}
        overlay = np.zeros((*a.shape, 3), dtype="uint8")
        overlay[a & b] = [240, 240, 240]
        overlay[a & ~b] = [255, 90, 80]
        overlay[b & ~a] = [60, 200, 255]
        Image.fromarray(overlay).resize((a.shape[1] * 8, a.shape[0] * 8), Image.Resampling.NEAREST).save(args.out / "topology-diff.png")
    else:
        report["topology"] = {"comparable": False, "reason": "Different detected axis counts; inspect masks/crops or threshold."}
    (args.out / "metrics.json").write_text(json.dumps(report, indent=2) + "\n")
    summary = {name: {key: value for key, value in report[name].items() if key not in ("x_centres", "y_centres")} for name in "ab"}
    summary["topology"] = report["topology"]
    summary["font_metrics_fit"] = report["font_metrics_fit"]
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
