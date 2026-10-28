//! The 3D route (Playroom P9's twin): Lumen's GLB emitter and
//! the renders-of-the-3D path, at teaching scale.
//!
//! The estate's ruled 3D path is: author in 3D, render to 2D,
//! and gate on determinism — the same scene, the same seed,
//! renders that twin-run **byte-identical**. This module is
//! that path in miniature, pure Rust and dependency-free:
//!
//! - a fixed scene (a unit cube, the "performer block"),
//! - an orbit determined by a step count (the seed's role),
//! - a software rasterizer producing an RGB frame,
//! - writers for the two artifacts: a binary glTF (`.glb`)
//!   of the scene, and a PNG of the render (hand-rolled:
//!   stored-block DEFLATE, because this workspace carries no
//!   compression dependency and needs none).
//!
//! Everything is integer or fixed-order float math over
//! explicit inputs, so two runs produce the same bytes — the
//! twin-run hash is the level's pass gate, exactly as in P4.

/// A 3D point/vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    /// X.
    pub x: f64,
    /// Y.
    pub y: f64,
    /// Z.
    pub z: f64,
}

impl Vec3 {
    /// Rotate around the Y (up) axis by `turns` of a full
    /// revolution.
    #[must_use]
    pub fn rot_y(self, turns: f64) -> Self {
        let (s, c) = (turns * std::f64::consts::TAU).sin_cos();
        Self {
            x: self.x * c + self.z * s,
            y: self.y,
            z: -self.x * s + self.z * c,
        }
    }
}

/// The scene: one unit cube centered at the origin, 8 vertices,
/// 12 triangles. Fixed by construction — the scene is data the
/// seed animates, never the other way round.
#[must_use]
pub fn cube_vertices() -> [Vec3; 8] {
    let h = 0.5;
    [
        Vec3 {
            x: -h,
            y: -h,
            z: -h,
        },
        Vec3 { x: h, y: -h, z: -h },
        Vec3 { x: h, y: h, z: -h },
        Vec3 { x: -h, y: h, z: -h },
        Vec3 { x: -h, y: -h, z: h },
        Vec3 { x: h, y: -h, z: h },
        Vec3 { x: h, y: h, z: h },
        Vec3 { x: -h, y: h, z: h },
    ]
}

/// Triangle indices into [`cube_vertices`], wound outward.
pub const CUBE_TRIANGLES: [[usize; 3]; 12] = [
    [0, 1, 2],
    [0, 2, 3], // back
    [5, 4, 7],
    [5, 7, 6], // front
    [4, 0, 3],
    [4, 3, 7], // left
    [1, 5, 6],
    [1, 6, 2], // right
    [3, 2, 6],
    [3, 6, 7], // top
    [4, 5, 1],
    [4, 1, 0], // bottom
];

/// Per-face flat shades (0..=255 greyscale) — the toon-shading
/// contract in miniature: each face takes exactly one shade
/// from a small ramp (House Cel is a *shading* contract in 3D:
/// hard bands, no gradients).
pub const FACE_SHADES: [u8; 12] = [
    90, 90, // back
    200, 200, // front
    130, 130, // left
    160, 160, // right
    230, 230, // top
    60, 60, // bottom
];

/// One projected triangle in the painter's sort: its face
/// index plus its three projected vertices `(x, y, depth)`.
type ProjectedTri = (usize, [(f64, f64, f64); 3]);

/// A rendered frame: RGB pixels, row-major, `width * height`.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// RGB triplets.
    pub pixels: Vec<u8>,
}

impl Frame {
    /// Render the scene at orbit step `step` of `steps` (the
    /// orbit angle is step/steps of a revolution — the seed
    /// made visible). Orthographic projection, painter's
    /// algorithm sorted by mean depth, flat shades. Pure: the
    /// same arguments always produce the same frame.
    #[must_use]
    pub fn render(width: usize, height: usize, step: u64, steps: u64) -> Self {
        let mut pixels = vec![0u8; width * height * 3];
        let mut depth = vec![f64::NEG_INFINITY; width * height];
        // Orbit phase wraps: step == steps is the same pose
        // as step 0, exactly (the phase is taken modulo the
        // orbit before it becomes an angle).
        let turns = if steps == 0 {
            0.0
        } else {
            (step % steps) as f64 / steps as f64
        };
        let verts: Vec<Vec3> = cube_vertices().iter().map(|v| v.rot_y(turns)).collect();
        // Project: screen x = world x, screen y = world y,
        // depth = world z (larger z toward the viewer). Scale
        // so the cube's unit width fills 80% of the frame.
        let scale = 0.8 * width.min(height) as f64;
        let project = |v: &Vec3| -> (f64, f64, f64) {
            (
                width as f64 / 2.0 + v.x * scale,
                height as f64 / 2.0 - v.y * scale,
                v.z,
            )
        };
        let mut tris: Vec<ProjectedTri> = CUBE_TRIANGLES
            .iter()
            .enumerate()
            .map(|(i, t)| {
                (
                    i,
                    [
                        project(&verts[t[0]]),
                        project(&verts[t[1]]),
                        project(&verts[t[2]]),
                    ],
                )
            })
            .collect();
        // Painter's order: farthest mean depth first.
        tris.sort_by(|a, b| {
            let da = (a.1[0].2 + a.1[1].2 + a.1[2].2) / 3.0;
            let db = (b.1[0].2 + b.1[1].2 + b.1[2].2) / 3.0;
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
        for (face, tri) in &tris {
            let shade = FACE_SHADES[*face];
            rasterize(&mut pixels, &mut depth, width, height, tri, shade);
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    /// FNV-1a hash of the frame bytes — the twin-run gate's
    /// comparison value.
    #[must_use]
    pub fn hash(&self) -> u64 {
        fnv1a(&self.pixels)
    }
}

/// Fill one projected triangle, z-buffered per pixel.
fn rasterize(
    pixels: &mut [u8],
    depth: &mut [f64],
    width: usize,
    height: usize,
    tri: &[(f64, f64, f64); 3],
    shade: u8,
) {
    let (x0, y0) = (tri[0].0, tri[0].1);
    let (x1, y1) = (tri[1].0, tri[1].1);
    let (x2, y2) = (tri[2].0, tri[2].1);
    let min_x = x0.min(x1).min(x2).floor().max(0.0) as usize;
    let max_x = x0.max(x1).max(x2).ceil().min(width as f64 - 1.0) as usize;
    let min_y = y0.min(y1).min(y2).floor().max(0.0) as usize;
    let max_y = y0.max(y1).max(y2).ceil().min(height as f64 - 1.0) as usize;
    let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
    if denom.abs() < f64::EPSILON || width == 0 || height == 0 {
        return;
    }
    for y in min_y..=max_y.min(height - 1) {
        for x in min_x..=max_x.min(width - 1) {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;
            let w0 = ((y1 - y2) * (px - x2) + (x2 - x1) * (py - y2)) / denom;
            let w1 = ((y2 - y0) * (px - x2) + (x0 - x2) * (py - y2)) / denom;
            let w2 = 1.0 - w0 - w1;
            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let z = w0 * tri[0].2 + w1 * tri[1].2 + w2 * tri[2].2;
                let idx = y * width + x;
                if z > depth[idx] {
                    depth[idx] = z;
                    pixels[idx * 3] = shade;
                    pixels[idx * 3 + 1] = shade;
                    pixels[idx * 3 + 2] = shade;
                }
            }
        }
    }
}

/// FNV-1a 64-bit — the workspace's tiny deterministic hash.
#[must_use]
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Serialize the scene as a binary glTF (GLB): a JSON chunk
/// describing one mesh with POSITION + index accessors, and a
/// BIN chunk carrying the interleaved vertex floats and index
/// bytes. Deterministic by construction (fixed key order,
/// fixed floats).
#[must_use]
pub fn scene_glb() -> Vec<u8> {
    let verts = cube_vertices();
    let mut bin: Vec<u8> = Vec::new();
    for v in &verts {
        for f in [v.x, v.y, v.z] {
            bin.extend_from_slice(&(f as f32).to_le_bytes());
        }
    }
    let index_offset = bin.len();
    for t in &CUBE_TRIANGLES {
        for i in t {
            bin.extend_from_slice(&(*i as u16).to_le_bytes());
        }
    }
    let json = format!(
        "{{\"asset\":{{\"version\":\"2.0\",\"generator\":\"hands-on render3d\"}},\
         \"scene\":0,\"scenes\":[{{\"nodes\":[0]}}],\"nodes\":[{{\"mesh\":0}}],\
         \"meshes\":[{{\"primitives\":[{{\"attributes\":{{\"POSITION\":0}},\"indices\":1}}]}}],\
         \"accessors\":[{{\"bufferView\":0,\"componentType\":5126,\"count\":8,\"type\":\"VEC3\"}},\
         {{\"bufferView\":1,\"componentType\":5123,\"count\":36,\"type\":\"SCALAR\"}}],\
         \"bufferViews\":[{{\"buffer\":0,\"byteOffset\":0,\"byteLength\":{}}},\
         {{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{}}}],\
         \"buffers\":[{{\"byteLength\":{}}}]}}",
        8 * 12,
        index_offset,
        36 * 2,
        bin.len()
    );
    let mut glb: Vec<u8> = Vec::new();
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    let json_bytes = json.as_bytes();
    let json_padded = json_bytes.len().div_ceil(4) * 4;
    let bin_padded = bin.len().div_ceil(4) * 4;
    let total = 12 + 8 + json_padded + 8 + bin_padded;
    glb.extend_from_slice(&(total as u32).to_le_bytes());
    glb.extend_from_slice(&(json_padded as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(json_bytes);
    glb.resize(12 + 8 + json_padded, b' ');
    glb.extend_from_slice(&(bin_padded as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(&bin);
    glb.resize(total, 0);
    glb
}

/// Encode a frame as a PNG, hand-rolled: IHDR (8-bit RGB),
/// one IDAT carrying a zlib stream of stored (uncompressed)
/// DEFLATE blocks over filter-0 scanlines, IEND. Stored
/// blocks cap at 65535 bytes; larger frames chain them.
/// Correct PNG, minimal machinery — and no dependency.
#[must_use]
pub fn frame_png(frame: &Frame) -> Vec<u8> {
    let mut raw: Vec<u8> = Vec::with_capacity((frame.width * 3 + 1) * frame.height);
    for y in 0..frame.height {
        raw.push(0); // filter: none
        let row = &frame.pixels[y * frame.width * 3..(y + 1) * frame.width * 3];
        raw.extend_from_slice(row);
    }
    // zlib: header 0x78 0x01 (no/low compression), stored
    // blocks, Adler-32 of the raw data.
    let mut z: Vec<u8> = vec![0x78, 0x01];
    let mut chunks = raw.chunks(65535).peekable();
    if raw.is_empty() {
        z.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    }
    while let Some(chunk) = chunks.next() {
        let last = chunks.peek().is_none();
        z.push(if last { 1 } else { 0 });
        let len = chunk.len() as u16;
        z.extend_from_slice(&len.to_le_bytes());
        z.extend_from_slice(&(!len).to_le_bytes());
        z.extend_from_slice(chunk);
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());
    let mut png: Vec<u8> = Vec::new();
    png.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
    let mut ihdr: Vec<u8> = Vec::new();
    ihdr.extend_from_slice(&(frame.width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(frame.height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit, RGB, deflate, std filters, no interlace
    write_chunk(&mut png, b"IHDR", &ihdr);
    write_chunk(&mut png, b"IDAT", &z);
    write_chunk(&mut png, b"IEND", &[]);
    png
}

fn write_chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let mut crc_data: Vec<u8> = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(kind);
    crc_data.extend_from_slice(data);
    png.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

/// CRC-32 (IEEE, as PNG uses).
#[must_use]
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xffff_ffff;
    for b in bytes {
        crc ^= u32::from(*b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Adler-32 (as zlib uses).
#[must_use]
pub fn adler32(bytes: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for byte in bytes {
        a = (a + u32::from(*byte)) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_are_byte_identical_across_runs() {
        let a = Frame::render(48, 48, 3, 16);
        let b = Frame::render(48, 48, 3, 16);
        assert_eq!(a, b);
        assert_eq!(a.hash(), b.hash());
    }

    #[test]
    fn the_orbit_changes_the_picture() {
        let a = Frame::render(48, 48, 0, 16);
        let b = Frame::render(48, 48, 4, 16);
        assert_ne!(a.hash(), b.hash());
        let back = Frame::render(48, 48, 16, 16);
        assert_eq!(
            a.hash(),
            back.hash(),
            "a full turn returns to the start frame"
        );
    }

    #[test]
    fn the_render_is_not_blank_and_uses_flat_shades() {
        let f = Frame::render(48, 48, 0, 16);
        let lit = f.pixels.iter().filter(|p| **p > 0).count();
        assert!(lit > 100, "the cube covers real area");
        let mut shades: Vec<u8> = f.pixels.iter().copied().filter(|p| *p > 0).collect();
        shades.sort_unstable();
        shades.dedup();
        assert!(shades.iter().all(|s| FACE_SHADES.contains(s)));
    }

    #[test]
    fn glb_has_the_gltf_envelope() {
        let glb = scene_glb();
        assert_eq!(&glb[0..4], b"glTF");
        let total = u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize;
        assert_eq!(total, glb.len());
        let text = String::from_utf8_lossy(&glb);
        assert!(text.contains("\"asset\":{\"version\":\"2.0\""));
        assert_eq!(scene_glb(), glb, "GLB serialization is deterministic");
    }

    #[test]
    fn png_has_the_png_envelope_and_crc() {
        let f = Frame::render(16, 16, 1, 8);
        let png = frame_png(&f);
        assert_eq!(
            &png[0..8],
            &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]
        );
        assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
        // The classic CRC check value.
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
        assert_eq!(frame_png(&f), png, "PNG bytes are deterministic");
    }

    // ---- adversarial ----

    #[test]
    fn degenerate_frames_do_not_panic() {
        let f = Frame::render(0, 0, 0, 0);
        assert_eq!(f.pixels.len(), 0);
        let _ = frame_png(&f);
        let g = Frame::render(4, 4, 0, 0); // zero steps: no orbit
        assert!(g.pixels.iter().any(|p| *p > 0));
    }

    #[test]
    fn large_frames_chain_stored_blocks() {
        // 200x200 RGB = 120_000 raw bytes > 65535: the zlib
        // stream must chain stored blocks and stay valid.
        let f = Frame::render(200, 200, 2, 16);
        let png = frame_png(&f);
        // IDAT length field sits after signature(8) + the
        // whole IHDR chunk(4 len + 4 type + 13 data + 4 crc).
        let idat_len = u32::from_be_bytes(png[33..37].try_into().unwrap()) as usize;
        assert!(idat_len > 65535);
        assert_eq!(&png[37..41], b"IDAT");
    }
}
