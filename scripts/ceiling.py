"""Offline ocean ceilings under Abiogenesis scoring: a guess earns 4 - popcount(guess ^ byte)
(bits right minus 4). Usage: ceiling.py NAME FILE [START LEN] ...

Predictors, all deterministic single guesses:
  free_bits    bitwise-majority constant: what you get for knowing nothing but the alphabet
  o1_last      the 4-op learner (store load guess dive): what followed this byte last time
  o1_ceiling   best possible order-1 table (bitwise majority per context, in-sample)
  o2_last      last successor of the previous two bytes (needs a 64 KB memory)
  o2_ceiling   best order-2 table, in-sample
  o3_last      last successor of the previous three bytes (16 MB memory)
"""
import sys
import numpy as np

POP = np.array([bin(i).count("1") for i in range(256)], dtype=np.int8)


def score(pred, tgt):
    s = 4 - POP[np.bitwise_xor(pred.astype(np.uint8), tgt)]
    return float(s.mean()), float((pred == tgt).mean())


def last_successor(ctx, tgt):
    order = np.argsort(ctx, kind="stable")
    c, t = ctx[order], tgt[order]
    prev = np.zeros_like(t)
    same = c[1:] == c[:-1]
    prev[1:] = np.where(same, t[:-1], 0)
    pred = np.empty_like(t)
    pred[order] = prev
    return pred


def bit_majority(ctx, tgt, nctx):
    pred = np.zeros(len(tgt), dtype=np.uint8)
    n = np.bincount(ctx, minlength=nctx)
    for k in range(8):
        ones = np.bincount(ctx, weights=(tgt >> k) & 1, minlength=nctx)
        pred |= ((ones * 2 > n)[ctx].astype(np.uint8) << k)
    return pred


def run(name, b):
    tgt = b[3:]
    p1 = b[2:-1].astype(np.int64)
    p2 = b[1:-2].astype(np.int64)
    p3 = b[:-3].astype(np.int64)
    c1, c2, c3 = p1, p2 << 8 | p1, p3 << 16 | p2 << 8 | p1
    rows = {
        "free_bits": bit_majority(np.zeros(len(tgt), dtype=np.int64), tgt, 1),
        "o1_last": last_successor(c1, tgt),
        "o1_ceiling": bit_majority(c1, tgt, 256),
        "o2_last": last_successor(c2, tgt),
        "o2_ceiling": bit_majority(c2, tgt, 1 << 16),
        "o3_last": last_successor(c3, tgt),
    }
    res = {k: score(v, tgt) for k, v in rows.items()}
    base = res["free_bits"][0]
    print(f"\n{name}  ({len(b):,} bytes, {len(np.unique(b))} distinct)")
    print(f"  {'predictor':<11} {'bits/guess':>10} {'above free':>10} {'exact':>7}")
    for k, (s, e) in res.items():
        print(f"  {k:<11} {s:>10.3f} {s - base:>10.3f} {e:>7.1%}")


args = sys.argv[1:]
while args:
    name, path = args[0], args[1]
    args = args[2:]
    data = np.fromfile(path, dtype=np.uint8)
    if args and args[0].isdigit():
        start, n = int(args[0]), int(args[1])
        args = args[2:]
        data = data[start:start + n]
    run(name, data)
