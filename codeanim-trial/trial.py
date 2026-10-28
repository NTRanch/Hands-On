#!/usr/bin/env python3
"""Codeanim trial on Opal (Hands-On, House Cel) — 2026-10-26.

Extends the estate codeanim toolchain (ntr-corpus-skeleton/codeanim/
animate.py: fixed-step springs, seeded closed-form motion, twin-run
byte-identity) from whole-still warps to LAYERED parts: Opal's stage
SVG from game.html is decomposed by svgmini.py into 18 raster layers
with pivots, and three motion classes are evaluated on a fixed 16 fps
clock:

  idle    - breath / blink / look drift / hair+tail sway (loop)
  climb   - expression ladder Neutral->Smiling->Flushed->Aroused->
            Torogao as continuous parameter travel (transition)
  milking - rhythmic compress-release, spring-fitted chest
            follow-through, tail counter-motion (loop, stress test)

Everything is a closed-form function of frame index or a fixed-dt
(240 Hz) spring simulation warmed into its periodic steady state
(the spike's method) — no wall clock, no unseeded RNG. The whole
pipeline runs TWICE; sha256 of every output must match (the spike
standard: params + code == the clip, regenerable exactly).
"""
import hashlib
import json
import math
import sys
import time
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, str(Path(__file__).parent))
from svgmini import Rasterizer  # noqa: E402

HERE = Path(__file__).parent
W, H = 800, 520
FPS = 16
SIM_DT = 1.0 / 240.0

# ---------------------------------------------------------------- layers

LAYER_ORDER = [
    "bg", "rear_body", "rear_tail", "bucket", "front_outfit", "chest",
    "handL", "handR", "head_base", "head_hair", "face_blush",
    "face_eyes_open", "face_eyes_half", "face_eyes_up", "face_eyes_melt",
    "face_sparkles", "face_mouth_smile", "face_mouth_open", "face_hearts",
]
FACE_IDS = {"blush", "eyesOpen", "eyesHalf", "eyesUp", "eyesMelt",
            "sparkles", "mouthSmile", "mouthOpen", "hearts"}
HAIR_D = ("M538 238", "M600 162", "M548 200")


def _anc_ids(ancestors):
    return [a.get("id") for a in ancestors]


def build_layers():
    r = Rasterizer(str(HERE / "stage.svg"), W, H,
                   strip_opacity_ids=("blush", "eyesHalf", "eyesUp", "eyesMelt",
                                      "sparkles", "mouthOpen", "hearts",
                                      "handL", "handR"))

    def render(pred):
        return r.render(pred)

    def under(layer_root):
        return lambda el, anc: layer_root in _anc_ids(anc)

    layers = {}
    # sky/grass backdrop is CSS in the game; reproduce it as layer zero
    sky = Image.new("RGBA", (W, H), (124, 199, 255, 255))
    ImageDraw.Draw(sky).rectangle([0, 322, W, H], fill=(121, 216, 79, 255))
    layers["sky"] = sky
    layers["bg"] = render(lambda el, anc: not ({"rearView", "frontView"} & set(_anc_ids(anc)))
                          and el.tag.split("}")[-1] in ("line", "rect")
                          and not any(a.get("clip-path") for a in anc))
    layers["rear_body"] = render(lambda el, anc: "rearView" in _anc_ids(anc)
                                 and "tail" not in _anc_ids(anc))
    layers["rear_tail"] = render(under("tail"))
    layers["bucket"] = render(lambda el, anc: el.tag.split("}")[-1] == "path"
                              and (el.get("d") or "").startswith("M150 500"))
    layers["front_outfit"] = render(lambda el, anc: "frontView" in _anc_ids(anc)
                                    and not ({"chest", "handL", "handR", "head"} & set(_anc_ids(anc))))
    layers["chest"] = render(under("chest"))
    layers["handL"] = render(under("handL"))
    layers["handR"] = render(under("handR"))

    def head_pred(want):
        def pred(el, anc):
            ids = _anc_ids(anc)
            if "head" not in ids:
                return False
            face_anc = [i for i in ids if i in FACE_IDS]
            d = el.get("d") or ""
            is_hair = el.tag.split("}")[-1] == "path" and d.startswith(HAIR_D)
            if want == "face_blush":
                return "blush" in ids
            if want == "face_eyes_open":
                return "eyesOpen" in ids
            if want == "face_eyes_half":
                return "eyesHalf" in ids
            if want == "face_eyes_up":
                return "eyesUp" in ids
            if want == "face_eyes_melt":
                return "eyesMelt" in ids
            if want == "face_sparkles":
                return "sparkles" in ids
            if want == "face_mouth_open":
                return "mouthOpen" in ids
            if want == "face_hearts":
                return "hearts" in ids
            if want == "face_mouth_smile":
                return el.get("id") == "mouthSmile"
            if want == "head_hair":
                return is_hair and not face_anc
            if want == "head_base":
                return not face_anc and not is_hair
            return False
        return pred

    for name in ("head_base", "head_hair", "face_blush", "face_eyes_open",
                 "face_eyes_half", "face_eyes_up", "face_eyes_melt",
                 "face_sparkles", "face_mouth_smile", "face_mouth_open",
                 "face_hearts"):
        layers[name] = render(head_pred(name))
    return layers


# ---------------------------------------------------------------- math

def sstep(x):
    x = min(1.0, max(0.0, x))
    return x * x * (3.0 - 2.0 * x)


def seg(s, a, b):
    return sstep((s - a) / (b - a))


def mat3(pivot=(0.0, 0.0), sx=1.0, sy=1.0, rot_deg=0.0, tx=0.0, ty=0.0):
    th = math.radians(rot_deg)
    co, si = math.cos(th), math.sin(th)
    # F = T(t) * T(pivot) * R * S * T(-pivot)
    a = co * sx
    b = si * sx
    c = -si * sy
    d = co * sy
    px, py = pivot
    e = px - (a * px + c * py) + tx
    f = py - (b * px + d * py) + ty
    return np.array([[a, c, e], [b, d, f], [0.0, 0.0, 1.0]])


def xform(img, m):
    if m is None:
        return img
    inv = np.linalg.inv(m)
    coeffs = (inv[0, 0], inv[0, 1], inv[0, 2], inv[1, 0], inv[1, 1], inv[1, 2])
    return img.transform((W, H), Image.AFFINE, coeffs, resample=Image.BILINEAR)


def set_opacity(img, w):
    if w >= 0.999:
        return img
    if w <= 0.001:
        return None
    arr = np.asarray(img).astype(np.float32)
    arr[..., 3] = np.rint(arr[..., 3] * w)
    return Image.fromarray(arr.astype(np.uint8), "RGBA")


def warp(img, cx, cy, rx, ry, dx, dy):
    """Spike-style elliptical smoothstep-falloff bilinear warp (RGBA)."""
    arr = np.asarray(img).astype(np.float32)
    pad = 16
    arrp = np.pad(arr, ((pad, pad), (pad, pad), (0, 0)), mode="reflect")
    gy, gx = np.mgrid[0:H, 0:W].astype(np.float32)
    r = np.sqrt(((gx - cx) / rx) ** 2 + ((gy - cy) / ry) ** 2)
    fall = (1.0 - (r * r * (3.0 - 2.0 * r))).astype(np.float32)
    fall[r >= 1.0] = 0.0
    fall[r <= 0.0] = 1.0
    sx = np.clip(gx - dx * fall + pad, 0, W + 2 * pad - 2)
    sy = np.clip(gy - dy * fall + pad, 0, H + 2 * pad - 2)
    x0 = np.floor(sx).astype(np.int32)
    y0 = np.floor(sy).astype(np.int32)
    fx = (sx - x0)[..., None]
    fy = (sy - y0)[..., None]
    Ia = arrp[y0, x0]
    Ib = arrp[y0, x0 + 1]
    Ic = arrp[y0 + 1, x0]
    Id = arrp[y0 + 1, x0 + 1]
    out = Ia * (1 - fx) * (1 - fy) + Ib * fx * (1 - fy) + Ic * (1 - fx) * fy + Id * fx * fy
    return Image.fromarray(np.rint(out).astype(np.uint8), "RGBA")


def spring1d(target_fn, period, n_frames, fps, k, c, warmup_periods=12):
    """Fixed-dt spring toward a periodic target; warmed into steady
    state (the spike's method). Returns (values[n_frames], seam
    residual). Duration must be an integer number of periods."""
    steps_per_frame = int(round(1.0 / SIM_DT / fps))
    x = v = 0.0
    t = 0.0
    for _ in range(int(round(warmup_periods * period / SIM_DT))):
        a = -k * (x - target_fn(t)) - c * v
        v += a * SIM_DT
        x += v * SIM_DT
        t += SIM_DT
    x0 = x
    vals = np.zeros(n_frames)
    for f in range(n_frames):
        vals[f] = x
        for _ in range(steps_per_frame):
            a = -k * (x - target_fn(t)) - c * v
            v += a * SIM_DT
            x += v * SIM_DT
            t += SIM_DT
    return vals, float(abs(x - x0))


def expr_weights(s):
    """The game's governed ladder as continuous weights.
    Ladder: 0 Neutral, 1 Smiling, 2 Flushed, 3 Aroused, 4 Torogao."""
    return {
        "face_eyes_open": 1.0 - seg(s, 1.0, 2.0),
        "face_eyes_half": seg(s, 1.0, 2.0) * (1.0 - seg(s, 2.0, 3.0)),
        "face_eyes_up": seg(s, 2.0, 3.0) * (1.0 - seg(s, 3.15, 4.0)),
        "face_eyes_melt": seg(s, 3.0, 4.0),
        "face_blush": seg(s, 0.8, 2.0),
        "face_sparkles": seg(s, 2.2, 3.2),
        "face_mouth_open": seg(s, 2.4, 3.2),
        "face_mouth_smile": 1.0 - seg(s, 2.4, 3.2),
        "face_hearts": seg(s, 3.4, 4.0),
    }


# ---------------------------------------------------------------- compose

class Stage:
    def __init__(self, layers):
        self.layers = layers

    def frame(self, spec):
        """spec: {layer: (matrix3x3 or None, opacity)} plus optional
        'chest_warp': (cx, cy, rx, ry, dx, dy) applied pre-transform."""
        canvas = self.layers["sky"].copy()
        chest_warp = spec.get("chest_warp")
        for name in LAYER_ORDER:
            entry = spec.get(name)
            if entry is None:
                # default: static full opacity for base layers, else skip
                if name in ("bg", "rear_body", "rear_tail", "bucket", "front_outfit",
                            "chest", "head_base", "head_hair", "face_eyes_open",
                            "face_mouth_smile"):
                    canvas.alpha_composite(self.layers[name])
                continue
            m, w = entry
            if w <= 0.001:
                continue
            img = self.layers[name]
            if name == "chest" and chest_warp is not None:
                img = warp(img, *chest_warp)
            if m is not None:
                img = xform(img, m)
            img = set_opacity(img, w)
            if img is not None:
                canvas.alpha_composite(img)
        return canvas.convert("RGB")


def head_spec(m_head, weights, gaze=(0.0, 0.0), extra=None):
    """Build the spec entries for the whole head stack sharing m_head."""
    spec = {}
    spec["head_base"] = (m_head, 1.0)
    spec["head_hair"] = (m_head, 1.0)
    for name, w in weights.items():
        m = m_head
        if name.startswith("face_eyes"):
            m = mat3(tx=gaze[0], ty=gaze[1]) @ m_head
        spec[name] = (m, w)
    if extra:
        spec.update(extra)
    return spec


# ---------------------------------------------------------------- motions

def motion_idle(n=64):
    tail_ang, tail_res = spring1d(
        lambda t: 6.0 * math.sin(2 * math.pi * t / 2.0 + 0.9),
        period=2.0, n_frames=n, fps=FPS, k=55.0, c=6.5)
    frames = []
    for f in range(n):
        t = f / FPS
        ph = 2 * math.pi * t / 4.0
        bob = 2.2 * math.sin(ph)
        bs = 1.0 + 0.006 * math.sin(ph + math.pi / 2)
        m_rear = mat3(pivot=(195, 520), sx=bs, sy=bs, tx=0, ty=bob)
        m_front = mat3(pivot=(600, 520), sx=bs, sy=bs, tx=0, ty=bob)
        m_chest = mat3(pivot=(600, 345), sx=1 + 0.010 * math.sin(ph + math.pi / 2),
                       sy=1 + 0.010 * math.sin(ph + math.pi / 2)) @ m_front
        blink = 0.0
        for start in (20, 50):
            if start <= f < start + 6:
                blink = math.sin(math.pi * (f - start) / 6.0)
        weights = {"face_eyes_open": 1.0 - 0.92 * blink,
                   "face_eyes_half": 0.92 * blink,
                   "face_eyes_up": 0.0, "face_eyes_melt": 0.0,
                   "face_blush": 0.0, "face_sparkles": 0.0,
                   "face_mouth_open": 0.0, "face_mouth_smile": 1.0,
                   "face_hearts": 0.0}
        gaze = (2.0 * math.sin(ph + 1.2), 0.6 * math.sin(ph + 1.2))
        m_head = mat3(pivot=(600, 520), sx=bs, sy=bs, tx=0.8 * math.sin(ph + 1.2), ty=bob)
        m_hair = mat3(pivot=(600, 168), rot_deg=1.5 * math.sin(ph - 0.7)) @ m_head
        spec = {
            "rear_body": (m_rear, 1.0),
            "rear_tail": (mat3(pivot=(262, 350), rot_deg=float(tail_ang[f])), 1.0),
            "front_outfit": (m_front, 1.0),
            "chest": (m_chest, 1.0),
            "handL": (None, 0.0), "handR": (None, 0.0),
        }
        spec.update(head_spec(m_head, weights, gaze=gaze))
        spec["head_hair"] = (m_hair, 1.0)
        # blush/sparkle scales anchored even at zero weight: skip
        frames.append(spec)
    params = {
        "motion": "idle", "frames": n, "fps": FPS, "loop": True,
        "breath": {"period_s": 4.0, "bob_px": 2.2, "scale_amp": 0.006,
                   "chest_micro_scale": 0.010},
        "blink": {"windows_frames": [[20, 26], [50, 56]], "depth": 0.92,
                  "profile": "sin(pi*u), eyeHalf as lid"},
        "look_drift": {"gaze_x_px": 2.0, "gaze_y_px": 0.6, "phase_rad": 1.2,
                       "period_s": 4.0},
        "hair_sway": {"amp_deg": 1.5, "phase_lag_rad": -0.7, "pivot": [600, 168]},
        "tail_spring": {"k": 55.0, "c": 6.5, "zeta": 0.44, "drive_amp_deg": 6.0,
                        "drive_period_s": 2.0, "phase_rad": 0.9,
                        "seam_residual_deg": round(tail_res, 6)},
        "expression_s": 0.0,
    }
    return frames, params


def motion_climb(n=96):
    tail_ang, _ = spring1d(
        lambda t: 3.0 * math.sin(2 * math.pi * t / 2.0 + 0.9),
        period=2.0, n_frames=n, fps=FPS, k=55.0, c=6.5)
    frames = []
    for f in range(n):
        t = f / FPS
        ph = 2 * math.pi * t / 4.0
        bob = 2.2 * math.sin(ph)
        raw = min(1.0, max(0.0, (f - 8) / 72.0)) * 4.0
        if raw >= 4.0:
            s = 4.0
        else:
            kk = int(math.floor(raw))
            s = kk + sstep(raw - kk)
        w = expr_weights(s)
        m_front = mat3(pivot=(600, 520), tx=0, ty=bob)
        m_chest = mat3(pivot=(600, 345), sx=1 + 0.012 * s, sy=1 + 0.012 * s) @ m_front
        m_head = mat3(pivot=(600, 302), rot_deg=-0.85 * s, tx=0, ty=bob - 1.6 * s)
        m_hair = mat3(pivot=(600, 168), rot_deg=1.5 * math.sin(ph - 0.7) + 0.5 * s) @ m_head
        pulse = math.sin(2 * math.pi * f / 8.0)
        extra = {
            "face_blush": (mat3(pivot=(600, 263), sx=1 + 0.06 * s / 4, sy=1 + 0.06 * s / 4) @ m_head,
                           w["face_blush"] * (0.8 + 0.2 * s / 4)),
            "face_sparkles": (mat3(pivot=(600, 235), sx=1 + 0.1 * pulse, sy=1 + 0.1 * pulse) @ m_head,
                              w["face_sparkles"] * (0.75 + 0.25 * pulse)),
            "face_mouth_open": (mat3(pivot=(600, 273), sx=1 + 0.18 * seg(s, 3, 4),
                                     sy=1 + 0.18 * seg(s, 3, 4)) @ m_head,
                                w["face_mouth_open"]),
            "face_hearts": (m_head, w["face_hearts"] * (0.85 + 0.15 * math.sin(2 * math.pi * f / 8.0 + 1))),
        }
        spec = {
            "rear_body": (mat3(pivot=(195, 520), tx=0, ty=bob), 1.0),
            "rear_tail": (mat3(pivot=(262, 350), rot_deg=float(tail_ang[f]) * (1 + 0.7 * s / 4)), 1.0),
            "front_outfit": (m_front, 1.0),
            "chest": (m_chest, 1.0),
            "handL": (None, 0.0), "handR": (None, 0.0),
        }
        # Gaze roll (Owner's named bar, the 9228588 ahegao grammar):
        # the iris travel that the eyeUp layer starts (cy -3.4) must
        # CONTINUE into the melt, whose authored iris sits at cy -0.5
        # and would otherwise un-roll the gaze at peak. A parametric
        # upward offset on all eye layers, growing with s, keeps the
        # roll monotonic into the melted face.
        gaze = (0.0, -3.2 * seg(s, 2.0, 4.0))
        spec.update(head_spec(m_head, w, gaze=gaze, extra=extra))
        spec["head_hair"] = (m_hair, 1.0)
        frames.append(spec)
    params = {
        "motion": "climb", "frames": n, "fps": FPS, "loop": False,
        "gaze_roll": "eye layers offset y by -3.2*seg(s,2,4) px so the "
                     "upward iris travel continues into the melt layer "
                     "instead of un-rolling at peak (9228588 grammar)",
        "s_curve": "s: 0 at f<8, linear-in-segments with smoothstep "
                   "intra-segment easing over f=8..79, hold 4.0 after",
        "ladder": ["Neutral", "Smiling", "Flushed", "Aroused", "Torogao"],
        "weights": "expr_weights(s): open 0-1, half 1-2, up 2-3, melt 3-4 "
                   "(smoothstep cross-fades); blush seg(0.8,2.0); sparkles "
                   "seg(2.2,3.2) + 8f pulse; mouth swap seg(2.4,3.2); "
                   "hearts seg(3.4,4.0)",
        "posture": {"head_tilt_deg_per_s": -0.85, "head_lift_px_per_s": -1.6,
                    "chest_fullness_scale_per_s": 0.012, "pivot_neck": [600, 302]},
        "tail_spring": {"k": 55.0, "c": 6.5, "amp_growth": "x(1+0.7*s/4)"},
    }
    return frames, params


PRESS = [0.15, 0.45, 0.80, 1.00, 1.00, 0.85, 0.50, 0.20]


def press_total(t):
    """Total press (both hands) as a continuous 0.5 s-periodic target."""
    phase = (t % 0.5) / 0.5 * 8.0
    i = int(math.floor(phase)) % 8
    j = (i + 1) % 8
    u = phase - math.floor(phase)
    return PRESS[i] * (1 - u) + PRESS[j] * u


def hand_state(f, hand):
    """(press, opacity) for one hand at frame f, loop-periodic mod 32."""
    best_age = None
    for slot in range(4):
        side = "L" if slot % 2 == 0 else "R"
        if side != hand:
            continue
        age = (f - slot * 8) % 32
        if best_age is None or age < best_age:
            best_age = age
    age = best_age
    if age < 8:
        p = PRESS[age]
        return p, 0.25 + 0.75 * min(1.0, p * 1.6)
    if age < 11:
        return 0.0, 0.55 * (1 - (age - 8) / 3.0)
    return 0.0, 0.0


def motion_milking(n=32):
    jig, jig_res = spring1d(press_total, period=0.5, n_frames=n, fps=FPS,
                            k=110.0, c=8.0)
    tail_ang, tail_res = spring1d(
        lambda t: -7.0 * press_total(t) + 3.5 * math.sin(2 * math.pi * t / 2.0 + 0.9),
        period=2.0, n_frames=n, fps=FPS, k=55.0, c=6.5)
    frames = []
    for f in range(n):
        t = f / FPS
        p_l, o_l = hand_state(f, "L")
        p_r, o_r = hand_state(f, "R")
        p_act = max(p_l, p_r)
        side_pivot = (562, 345) if p_l >= p_r else (638, 345)
        ph2 = 2 * math.pi * t / 2.0
        bob = 1.2 * math.sin(ph2 + 0.5)
        m_front = mat3(pivot=(600, 520), tx=0, ty=bob)
        m_chest = mat3(pivot=side_pivot, sx=1 - 0.045 * p_act, sy=1 - 0.06 * p_act) @ \
            mat3(pivot=(600, 345), sx=1.03, sy=1.03) @ m_front
        s_eff = 2.1 + 0.3 * p_act
        w = expr_weights(s_eff)
        m_head = mat3(pivot=(600, 302), rot_deg=-0.4 * p_act, tx=0, ty=bob + 1.8 * p_act)
        extra = {
            "face_blush": (mat3(pivot=(600, 263), sx=1 + 0.05 * p_act, sy=1 + 0.05 * p_act) @ m_head,
                           w["face_blush"]),
        }
        spec = {
            "rear_body": (mat3(pivot=(195, 470), rot_deg=0.6 * math.sin(ph2 - 0.8), tx=0, ty=bob), 1.0),
            "rear_tail": (mat3(pivot=(262, 350), rot_deg=float(tail_ang[f])), 1.0),
            "front_outfit": (m_front, 1.0),
            "chest": (m_chest, 1.0),
            "chest_warp": (side_pivot[0], 345, 105, 68, 1.5 * float(jig[f]), 5.0 * float(jig[f])),
            "handL": (mat3(tx=-12 * (1 - p_l) + 3 * p_l, ty=4 * (1 - p_l) - 1 * p_l), o_l),
            "handR": (mat3(tx=12 * (1 - p_r) - 3 * p_r, ty=4 * (1 - p_r) - 1 * p_r), o_r),
        }
        spec.update(head_spec(m_head, w, extra=extra))
        frames.append(spec)
    params = {
        "motion": "milking", "frames": n, "fps": FPS, "loop": True,
        "beat": "4 squeezes per 2 s loop, hands alternate L R L R every 8f",
        "press_profile_per_beat": PRESS,
        "press_profile_phases": ["approach", "approach", "compress", "compress",
                                 "hold", "release", "release", "tail"],
        "chest_squash": {"sx": -0.045, "sy": -0.06, "about": "pressed-side pivot"},
        "chest_jiggle_spring": {"k": 110.0, "c": 8.0, "zeta": 0.38,
                                "drive": "press_total(t), 0.5 s periodic",
                                "warp_amp_px": [1.5, 5.0],
                                "seam_residual": round(jig_res, 6)},
        "chest_fullness_frozen": 1.03,
        "tail_counter_spring": {"k": 55.0, "c": 6.5,
                                "drive": "-7deg*press_total + 3.5deg sway",
                                "seam_residual_deg": round(tail_res, 6)},
        "expression_s_base": 2.1, "expression_s_press_gain": 0.3,
        "breath_period_s": 2.0,
    }
    return frames, params


MOTIONS = {"idle": motion_idle, "climb": motion_climb, "milking": motion_milking}
SHEET_FRAMES = {"idle": [0, 10, 21, 32, 42, 53, 63],
                "climb": [0, 16, 32, 48, 64, 80, 88, 95],
                "milking": [0, 3, 5, 11, 13, 19, 21, 27]}


def seam_metrics(frames_rgb):
    arrs = [np.asarray(im).astype(np.float32) for im in frames_rgb]
    diffs = [float(np.mean(np.abs(arrs[i] - arrs[i - 1]))) for i in range(1, len(arrs))]
    seam = float(np.mean(np.abs(arrs[-1] - arrs[0])))
    return {"loop_seam_meanabsdiff": round(seam, 4),
            "median_interframe_meanabsdiff": round(float(np.median(diffs)), 4)}


def make_gif(frames_rgb, path):
    pal = frames_rgb[0].quantize(colors=256)
    q = [fr.quantize(palette=pal, dither=Image.Dither.NONE) for fr in frames_rgb]
    q[0].save(path, save_all=True, append_images=q[1:], duration=62, loop=0,
              optimize=False)


def make_sheet(frames_rgb, idxs, path, tile_w=400):
    tile_h = int(tile_w * H / W)
    cols = 4
    rows = math.ceil(len(idxs) / cols)
    sheet = Image.new("RGB", (cols * tile_w, rows * (tile_h + 16)), (24, 18, 28))
    d = ImageDraw.Draw(sheet)
    font = ImageFont.load_default()
    for i, fi in enumerate(idxs):
        fr = frames_rgb[fi].resize((tile_w, tile_h), Image.LANCZOS)
        x, y = (i % cols) * tile_w, (i // cols) * (tile_h + 16)
        sheet.paste(fr, (x, y))
        d.text((x + 6, y + tile_h + 3), f"f{fi:03d}", fill=(255, 240, 200), font=font)
    sheet.save(path)


def make_spritesheet(frames_rgb, path, cols=8):
    rows = math.ceil(len(frames_rgb) / cols)
    sheet = Image.new("RGB", (cols * W, rows * H), (0, 0, 0))
    for i, fr in enumerate(frames_rgb):
        sheet.paste(fr, ((i % cols) * W, (i // cols) * H))
    sheet.save(path, optimize=True)


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def render_run(stage, run_dir):
    run_dir.mkdir(parents=True, exist_ok=True)
    out = {}
    for name, fn in MOTIONS.items():
        specs, params = fn()
        mdir = run_dir / name
        mdir.mkdir(exist_ok=True)
        t0 = time.perf_counter()
        frames_rgb = []
        for i, spec in enumerate(specs):
            fr = stage.frame(spec)
            fr.save(mdir / f"f{i:03d}.png")
            frames_rgb.append(fr)
        wall = time.perf_counter() - t0
        make_gif(frames_rgb, mdir / f"{name}.gif")
        make_sheet(frames_rgb, SHEET_FRAMES[name], mdir / f"{name}_contact.png")
        make_spritesheet(frames_rgb, mdir / f"{name}_spritesheet.png")
        (mdir / f"{name}.params.json").write_text(json.dumps(params, indent=1))
        res = {"motion": name, "frames": len(frames_rgb), "fps": FPS,
               "render_wall_s": round(wall, 2), **seam_metrics(frames_rgb)}
        (mdir / f"{name}.results.json").write_text(json.dumps(res, indent=1))
        out[name] = res
        print(run_dir.name, name, res)
    return out


def main():
    parts_dir = HERE / "parts"
    parts_dir.mkdir(exist_ok=True)
    layers = build_layers()
    for name, img in layers.items():
        img.save(parts_dir / f"{name}.png")
    print("layers:", {k: (v.size, len(v.tobytes())) for k, v in list(layers.items())[:3]}, "...")
    stage = Stage(layers)
    results = {}
    for run in ("run1", "run2"):
        results[run] = render_run(stage, HERE / run)
    # determinism: hash every produced artifact in both runs.
    # *.results.json are excluded — they carry wall-clock render
    # times (logs, not artifacts; the spike's stated precedent).
    def hashes(run):
        base = HERE / run
        return {str(p.relative_to(base)): sha256(p)
                for p in sorted(base.rglob("*"))
                if p.is_file() and not p.name.endswith(".results.json")}
    h1, h2 = hashes("run1"), hashes("run2")
    (HERE / "hashes_run1.txt").write_text("\n".join(f"{v}  {k}" for k, v in h1.items()))
    (HERE / "hashes_run2.txt").write_text("\n".join(f"{v}  {k}" for k, v in h2.items()))
    same = h1 == h2
    (HERE / "determinism.txt").write_text(
        f"{'PASS' if same else 'FAIL'}: {sum(1 for k in h1 if h1.get(k) == h2.get(k))}"
        f"/{len(h1)} artifacts byte-identical across twin runs "
        f"(*.results.json excluded: wall-clock logs)\n")
    print("determinism:", same, len(h1), "files")
    (HERE / "results_all.json").write_text(json.dumps(results, indent=1))


if __name__ == "__main__":
    main()
