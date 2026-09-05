// Compute shader simulasi galaksi: akumulasi bintang + tonemap.
// Diadaptasi verbatim dr andromeda-simulation-tui (M20.8 follow-up plan).
//
// KEEP IN SYNC — rumus di file ini menduplikasi (harus identik dengan):
//   - orbit density-wave : src/galaxy_sim/orbit.rs          (fn eval, fn wrap_angle)
//   - proyeksi kamera    : src/galaxy_sim/camera.rs (Phase 6) (fn view_plane, fn to_pixel)
//   - tonemap            : src/galaxy_sim/render/tonemap.rs (fn tonemap_pixel)
//   - layout Star/Uniforms: src/galaxy_sim/model.rs

struct Star {
    a: f32,
    b: f32,
    tilt: f32,
    theta0: f32,
    omega: f32,
    z: f32,
    color: u32,  // R | G<<8 | B<<16 | weight<<24
    flags: u32,  // bit 0..1: anchor; bit 2: debu
}

struct Uniforms {
    time_myr: f32,
    pattern_omega: f32,
    zoom: f32,
    exposure: f32,
    view_center: vec2<f32>,
    screen: vec2<f32>,
    incl_cs: vec2<f32>,
    pa_cs: vec2<f32>,
    dust_k: vec4<f32>,           // xyz: ekstingsi debu; w: cell_aspect
    anchors: array<vec4<f32>, 3>,
    star_count: u32,
    tonemap_denom: f32,
    pad0: u32,
    pad1: u32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var<storage, read> stars: array<Star>;
// W*H*4 kanal (r, g, b, debu), fixed-point u32.
@group(0) @binding(2) var<storage, read_write> accum: array<atomic<u32>>;
// W*H piksel RGBA8 packed.
@group(0) @binding(3) var<storage, read_write> out_rgba: array<u32>;

const TAU: f32 = 6.2831853071795864769;
// 1.0 = satu bintang weight-16 warna-255 (lihat FIXED_NORM di model.rs).
const FIXED_NORM: f32 = 1.0 / (255.0 * 16.0);

fn wrap_angle(x: f32) -> f32 {
    return x - floor(x / TAU) * TAU;
}

@compute @workgroup_size(256)
fn accumulate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= u.star_count {
        return;
    }
    let s = stars[i];
    let anchor = s.flags & 3u;

    // orbit density-wave (mirror orbit.rs)
    let theta = wrap_angle(s.theta0 + s.omega * u.time_myr);
    var tilt = s.tilt;
    if anchor == 0u {
        tilt = wrap_angle(s.tilt + u.pattern_omega * u.time_myr);
    }
    let p = vec2<f32>(s.a * cos(theta), s.b * sin(theta));
    let cr = cos(tilt);
    let sr = sin(tilt);
    let xy = vec2<f32>(p.x * cr - p.y * sr, p.x * sr + p.y * cr);
    let world = u.anchors[anchor].xyz + vec3<f32>(xy.x, xy.y, s.z);

    // proyeksi miring + PA + zoom (mirror camera.rs)
    let vy = world.y * u.incl_cs.x - world.z * u.incl_cs.y;
    let q = vec2<f32>(
        world.x * u.pa_cs.x - vy * u.pa_cs.y,
        world.x * u.pa_cs.y + vy * u.pa_cs.x,
    ) - u.view_center;
    let px = q.x * u.zoom + u.screen.x * 0.5;
    let py = q.y * u.zoom * u.dust_k.w + u.screen.y * 0.5;
    if px < 0.0 || py < 0.0 || px >= u.screen.x || py >= u.screen.y {
        return;
    }

    let idx = (u32(py) * u32(u.screen.x) + u32(px)) * 4u;
    let weight = (s.color >> 24u) & 0xffu;
    if (s.flags & 4u) != 0u {
        atomicAdd(&accum[idx + 3u], weight);
    } else {
        atomicAdd(&accum[idx + 0u], (s.color & 0xffu) * weight);
        atomicAdd(&accum[idx + 1u], ((s.color >> 8u) & 0xffu) * weight);
        atomicAdd(&accum[idx + 2u], ((s.color >> 16u) & 0xffu) * weight);
    }
}

fn asinh_f(x: f32) -> f32 {
    return log(x + sqrt(x * x + 1.0));
}

@compute @workgroup_size(256)
fn tonemap(@builtin(global_invocation_id) gid: vec3<u32>) {
    let count = u32(u.screen.x) * u32(u.screen.y);
    let pix = gid.x;
    if pix >= count {
        return;
    }
    let base = pix * 4u;
    let r = atomicLoad(&accum[base + 0u]);
    let g = atomicLoad(&accum[base + 1u]);
    let b = atomicLoad(&accum[base + 2u]);
    let d = atomicLoad(&accum[base + 3u]);
    // self-clear untuk frame berikutnya (tanpa pass clear terpisah)
    atomicStore(&accum[base + 0u], 0u);
    atomicStore(&accum[base + 1u], 0u);
    atomicStore(&accum[base + 2u], 0u);
    atomicStore(&accum[base + 3u], 0u);

    // mirror tonemap.rs
    let dn = f32(d) / 16.0;
    let ext = exp(-u.dust_k.xyz * dn);
    let c = vec3<f32>(f32(r), f32(g), f32(b)) * FIXED_NORM * ext;
    let l = max(c.x, max(c.y, c.z));
    var o = vec3<f32>(0.0, 0.0, 0.0);
    if l > 0.0 {
        let lt = clamp(asinh_f(u.exposure * l) / u.tonemap_denom, 0.0, 1.0);
        o = clamp(c * (lt / l), vec3<f32>(0.0), vec3<f32>(1.0));
    }
    let ob = vec3<u32>(o * 255.0 + vec3<f32>(0.5));
    out_rgba[pix] = ob.x | (ob.y << 8u) | (ob.z << 16u) | 0xff000000u;
}
