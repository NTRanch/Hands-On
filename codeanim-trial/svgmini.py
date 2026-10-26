#!/usr/bin/env python3
"""svgmini — a tiny deterministic rasterizer for the SVG subset used by
game.html's stage art (flat House Cel shapes only).

Supported: path (M/m L/l H/h V/v Q/q C/c Z/z, fixed-subdivision curve
flattening), ellipse, circle, rect, line, g, use (defs resolution with
inherited fill/stroke context), transform (translate/scale/rotate),
opacity accumulation. NOT supported: text (skipped — stage labels are
annotations), gradients, filters, masks, clip paths (the one clipped
element in the stage, bucketFill, has height 0 at rest = invisible).

Rendering: supersample x2, PIL polygon/line/ellipse primitives,
LANCZOS downscale. Deterministic: same input -> same bytes on this
toolchain (PIL 10.2.0 / numpy 1.26.4). Edge fidelity vs a browser
rasterizer is ~1px on curves; the trial judges MOTION, and every
layer the animator moves is rendered by this same rasterizer, so
comparisons are internally exact.
"""
import math
import re
import xml.etree.ElementTree as ET

import numpy as np
from PIL import Image, ImageDraw

SS = 2  # supersample factor
QSEG = 16  # fixed subdivisions per quadratic segment
CSEG = 20  # fixed subdivisions per cubic segment

IDENT = (1.0, 0.0, 0.0, 1.0, 0.0, 0.0)  # a,b,c,d,e,f : x'=ax+cy+e, y'=bx+dy+f


def mat_mul(m1, m2):
    a1, b1, c1, d1, e1, f1 = m1
    a2, b2, c2, d2, e2, f2 = m2
    return (
        a1 * a2 + c1 * b2,
        b1 * a2 + d1 * b2,
        a1 * c2 + c1 * d2,
        b1 * c2 + d1 * d2,
        a1 * e2 + c1 * f2 + e1,
        b1 * e2 + d1 * f2 + f1,
    )


def mat_apply(m, x, y):
    a, b, c, d, e, f = m
    return (a * x + c * y + e, b * x + d * y + f)


def parse_transform(s):
    m = IDENT
    if not s:
        return m
    for name, args in re.findall(r"(translate|scale|rotate)\(([^)]*)\)", s):
        vals = [float(v) for v in re.split(r"[,\s]+", args.strip()) if v]
        if name == "translate":
            t = (1, 0, 0, 1, vals[0], vals[1] if len(vals) > 1 else 0.0)
        elif name == "scale":
            sx = vals[0]
            sy = vals[1] if len(vals) > 1 else sx
            t = (sx, 0, 0, sy, 0.0, 0.0)
        else:  # rotate, degrees, optional center
            th = math.radians(vals[0])
            rot = (math.cos(th), math.sin(th), -math.sin(th), math.cos(th), 0.0, 0.0)
            if len(vals) > 2:
                cx, cy = vals[1], vals[2]
                t = mat_mul((1, 0, 0, 1, cx, cy), mat_mul(rot, (1, 0, 0, 1, -cx, -cy)))
            else:
                t = rot
        m = mat_mul(m, t)
    return m


TOKEN_RE = re.compile(r"[MmLlHhVvQqCcZz]|[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?")


def flatten_path(d):
    """Return list of subpaths; each is (points[(x,y)...], closed bool)."""
    tokens = TOKEN_RE.findall(d)
    subpaths = []
    pts = []
    closed = False
    cur = (0.0, 0.0)
    start = (0.0, 0.0)
    cmd = None
    i = 0

    def flush():
        nonlocal pts, closed
        if pts:
            subpaths.append((pts, closed))
        pts = []
        closed = False

    def num():
        nonlocal i
        v = float(tokens[i])
        i += 1
        return v

    while i < len(tokens):
        tok = tokens[i]
        if tok.isalpha():
            cmd = tok
            i += 1
            if cmd in "Zz":
                if pts:
                    pts.append(start)
                closed = True
                flush()
                cur = start
                cmd = None
            continue
        if cmd is None:
            raise ValueError("path data without command")
        rel = cmd.islower()
        c = cmd.upper()
        if c == "M":
            x, y = num(), num()
            if rel:
                x, y = x + cur[0], y + cur[1]
            flush()
            cur = start = (x, y)
            pts = [cur]
            cmd = "l" if rel else "L"
        elif c == "L":
            x, y = num(), num()
            if rel:
                x, y = x + cur[0], y + cur[1]
            cur = (x, y)
            pts.append(cur)
        elif c == "H":
            x = num()
            cur = (x + cur[0] if rel else x, cur[1])
            pts.append(cur)
        elif c == "V":
            y = num()
            cur = (cur[0], y + cur[1] if rel else y)
            pts.append(cur)
        elif c == "Q":
            x1, y1, x, y = num(), num(), num(), num()
            if rel:
                x1, y1 = x1 + cur[0], y1 + cur[1]
                x, y = x + cur[0], y + cur[1]
            p0 = cur
            for s in range(1, QSEG + 1):
                t = s / QSEG
                mt = 1 - t
                pts.append((
                    mt * mt * p0[0] + 2 * mt * t * x1 + t * t * x,
                    mt * mt * p0[1] + 2 * mt * t * y1 + t * t * y,
                ))
            cur = (x, y)
        elif c == "C":
            x1, y1, x2, y2, x, y = num(), num(), num(), num(), num(), num()
            if rel:
                x1, y1 = x1 + cur[0], y1 + cur[1]
                x2, y2 = x2 + cur[0], y2 + cur[1]
                x, y = x + cur[0], y + cur[1]
            p0 = cur
            for s in range(1, CSEG + 1):
                t = s / CSEG
                mt = 1 - t
                pts.append((
                    mt**3 * p0[0] + 3 * mt * mt * t * x1 + 3 * mt * t * t * x2 + t**3 * x,
                    mt**3 * p0[1] + 3 * mt * mt * t * y1 + 3 * mt * t * t * y2 + t**3 * y,
                ))
            cur = (x, y)
        else:
            raise ValueError(f"unsupported path command {cmd}")
    flush()
    return subpaths


def parse_color(s, opacity):
    if s is None or s == "none":
        return None
    if s.startswith("#") and len(s) == 7:
        r, g, b = int(s[1:3], 16), int(s[3:5], 16), int(s[5:7], 16)
        return (r, g, b, int(round(255 * opacity)))
    raise ValueError(f"unsupported color {s}")


class Ctx:
    def __init__(self, fill="#000000", stroke="none", stroke_width=1.0, opacity=1.0, matrix=IDENT):
        self.fill = fill
        self.stroke = stroke
        self.stroke_width = stroke_width
        self.opacity = opacity
        self.matrix = matrix

    def derive(self, el):
        c = Ctx(self.fill, self.stroke, self.stroke_width, self.opacity, self.matrix)
        if el.get("fill") is not None:
            c.fill = el.get("fill")
        if el.get("stroke") is not None:
            c.stroke = el.get("stroke")
        if el.get("stroke-width") is not None:
            c.stroke_width = float(el.get("stroke-width"))
        if el.get("opacity") is not None:
            c.opacity = c.opacity * float(el.get("opacity"))
        if el.get("transform") is not None:
            c.matrix = mat_mul(c.matrix, parse_transform(el.get("transform")))
        return c


class Rasterizer:
    def __init__(self, svg_path, width=800, height=520, strip_opacity_ids=()):
        text = open(svg_path).read()
        text = text.replace('<svg ', '<svg xmlns="http://www.w3.org/2000/svg" ', 1)
        self.root = ET.fromstring(text)
        self.W, self.H = width, height
        self.by_id = {}
        for el in self.root.iter():
            if el.get("id"):
                self.by_id[el.get("id")] = el
        # Layer roots whose game-side opacity is driven at runtime by
        # the game's state machine: as stills they must render at full
        # opacity; the animator owns their weight.
        for oid in strip_opacity_ids:
            if oid in self.by_id and self.by_id[oid].get("opacity") is not None:
                del self.by_id[oid].attrib["opacity"]
        self._canvas = None

    def render(self, layer_pred):
        """Render only elements for which layer_pred(el, ancestors) is
        True. Returns an RGBA Image at (W,H)."""
        self._canvas = Image.new("RGBA", (self.W * SS, self.H * SS), (0, 0, 0, 0))
        self._walk(self.root, Ctx(), [], layer_pred)
        out = self._canvas.resize((self.W, self.H), Image.LANCZOS)
        self._canvas = None
        return out

    def _walk(self, el, ctx, ancestors, pred):
        tag = el.tag.split("}")[-1]
        if tag in ("defs", "clipPath", "text"):
            return
        ctx = ctx.derive(el)
        if tag == "use":
            href = el.get("href") or el.get("{http://www.w3.org/1999/xlink}href")
            if href and href.startswith("#"):
                target = self.by_id[href[1:]]
                self._walk(target, ctx, ancestors + [el], pred)
            return
        if tag in ("path", "ellipse", "circle", "rect", "line"):
            if pred(el, ancestors):
                self._draw_shape(el, tag, ctx)
            return
        for child in el:
            self._walk(child, ctx, ancestors + [el], pred)

    def _draw_shape(self, el, tag, ctx):
        # Work in supersampled space: scale the composed matrix once,
        # then feed it RAW source coordinates (incl. use transforms).
        a, b, c, d, e, f = ctx.matrix
        m = (a * SS, b * SS, c * SS, d * SS, e * SS, f * SS)
        fill = parse_color(ctx.fill, ctx.opacity)
        stroke = parse_color(ctx.stroke, ctx.opacity)
        sw = ctx.stroke_width * SS
        # PIL ImageDraw replaces pixels; SVG paints source-over. Opaque
        # shapes may draw directly; anything translucent is drawn to a
        # temp image and alpha-composited (alpha-0 becomes a no-op
        # instead of an erase).
        translucent = (fill is not None and fill[3] < 255) or \
                      (stroke is not None and stroke[3] < 255)
        if translucent:
            tmp = Image.new("RGBA", self._canvas.size, (0, 0, 0, 0))
            draw = ImageDraw.Draw(tmp)
        else:
            draw = ImageDraw.Draw(self._canvas)
        if tag == "path":
            for pts, closed in flatten_path(el.get("d")):
                tp = [mat_apply(m, x, y) for (x, y) in pts]
                if fill is not None and len(tp) >= 3:
                    draw.polygon(tp, fill=fill)
                if stroke is not None and len(tp) >= 2:
                    line_pts = tp + [tp[0]] if closed else tp
                    draw.line(line_pts, fill=stroke, width=max(1, int(round(sw))), joint="curve")
                    if not closed:  # round caps
                        r = sw / 2.0
                        for (px, py) in (tp[0], tp[-1]):
                            draw.ellipse([px - r, py - r, px + r, py + r], fill=stroke)
        elif tag in ("ellipse", "circle"):
            cx = float(el.get("cx", 0))
            cy = float(el.get("cy", 0))
            if tag == "circle":
                rx = ry = float(el.get("r"))
            else:
                rx, ry = float(el.get("rx")), float(el.get("ry"))
            assert abs(m[1]) < 1e-9 and abs(m[2]) < 1e-9, "ellipse under rotation unsupported"
            ccx, ccy = mat_apply(m, cx, cy)
            rrx, rry = rx * abs(m[0]), ry * abs(m[3])
            box = [ccx - rrx, ccy - rry, ccx + rrx, ccy + rry]
            if fill is not None:
                draw.ellipse(box, fill=fill)
            if stroke is not None:
                draw.ellipse(box, outline=stroke, width=max(1, int(round(sw))))
        elif tag == "rect":
            x, y = float(el.get("x", 0)), float(el.get("y", 0))
            w, h = float(el.get("width")), float(el.get("height"))
            corners = [mat_apply(m, x, y), mat_apply(m, x + w, y),
                       mat_apply(m, x + w, y + h), mat_apply(m, x, y + h)]
            if fill is not None:
                draw.polygon(corners, fill=fill)
            if stroke is not None:
                draw.line(corners + [corners[0]], fill=stroke, width=max(1, int(round(sw))), joint="curve")
        elif tag == "line":
            p1 = mat_apply(m, float(el.get("x1")), float(el.get("y1")))
            p2 = mat_apply(m, float(el.get("x2")), float(el.get("y2")))
            if stroke is not None:
                draw.line([p1, p2], fill=stroke, width=max(1, int(round(sw))))
                r = sw / 2.0
                for (px, py) in (p1, p2):
                    draw.ellipse([px - r, py - r, px + r, py + r], fill=stroke)
        if translucent:
            self._canvas.alpha_composite(tmp)


def layer_image(rasterizer, keep_ids, keep_ancestor_ids=(), extra_pred=None):
    """Render one layer: elements whose own id is in keep_ids, or with
    an ancestor id in keep_ancestor_ids, filtered by extra_pred."""
    def pred(el, ancestors):
        if el.get("id") in keep_ids:
            return True
        if any(a.get("id") in keep_ancestor_ids for a in ancestors):
            return True
        if extra_pred is not None:
            return extra_pred(el, ancestors)
        return False
    return rasterizer.render(pred)


if __name__ == "__main__":
    import sys
    r = Rasterizer(sys.argv[1] if len(sys.argv) > 1 else "stage.svg")
    img = r.render(lambda el, anc: True)
    # backdrop like the game's CSS: sky to 62%, grass below
    bg = Image.new("RGBA", (800, 520), (124, 199, 255, 255))
    ImageDraw.Draw(bg).rectangle([0, 322, 800, 520], fill=(121, 216, 79, 255))
    bg.alpha_composite(img)
    bg.convert("RGB").save("rest_frame.png")
    print("rest frame -> rest_frame.png", bg.size)
