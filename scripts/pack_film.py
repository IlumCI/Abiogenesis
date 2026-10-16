"""Pack a frames directory (written with ABIO_FRAMES=DIR) into one film.bin for the viewer.

    pack_film.py FRAMES_DIR OUT [--max-mb 14] [--label TEXT] [--from EPOCH] [--to EPOCH]

Keeps evenly spaced frames so the file fits under --max-mb (an Artifact allows 15 MB per
file). Layout: b"ABFILM1\\0", frame count u32, label length u32, label (utf-8), then per
frame its length u32 and its bytes.
"""
import argparse
import pathlib
import struct

ap = argparse.ArgumentParser()
ap.add_argument("frames")
ap.add_argument("out")
ap.add_argument("--max-mb", type=float, default=14)
ap.add_argument("--label", default="")
ap.add_argument("--from", dest="lo", type=int, default=0)
ap.add_argument("--to", dest="hi", type=int, default=1 << 62)
a = ap.parse_args()

paths = sorted(p for p in pathlib.Path(a.frames).glob("*.abf") if a.lo <= int(p.stem) <= a.hi)
if not paths:
    raise SystemExit(f"no frames in {a.frames}")
sizes = [p.stat().st_size + 4 for p in paths]
budget = a.max_mb * 1e6 - 64 - len(a.label.encode())
step = 1
while sum(sizes[::step]) > budget:
    step += 1
keep = paths[::step]
if keep[-1] != paths[-1]:
    keep.append(paths[-1])
label = a.label.encode()
with open(a.out, "wb") as f:
    f.write(b"ABFILM1\0" + struct.pack("<II", len(keep), len(label)) + label)
    for p in keep:
        b = p.read_bytes()
        f.write(struct.pack("<I", len(b)) + b)
total = pathlib.Path(a.out).stat().st_size
print(f"{len(keep)} of {len(paths)} frames (every {step}), epochs {keep[0].stem.lstrip('0') or 0} to {keep[-1].stem.lstrip('0')}, {total / 1e6:.1f} MB")
