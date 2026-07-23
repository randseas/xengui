// SPDX-License-Identifier: Apache-2.0
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radius: f32,
    @location(3) blur: f32,
    @location(4) color: vec4<f32>,
    @location(5) inset: f32,
    @location(6) box_local_pos: vec2<f32>,
    @location(7) box_half_size: vec2<f32>,
    @location(8) box_radius: f32,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) blur: f32,
    @location(5) color: vec4<f32>,
    @location(6) inset: f32,
    @location(7) box_local_pos: vec2<f32>,
    @location(8) box_half_size: vec2<f32>,
    @location(9) box_radius: f32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.local_pos = local_pos;
    out.half_size = half_size;
    out.radius = radius;
    out.blur = blur;
    out.color = color;
    out.inset = inset;
    out.box_local_pos = box_local_pos;
    out.box_half_size = box_half_size;
    out.box_radius = box_radius;
    return out;
}

// Abramowitz-Stegun erf approximation, used for the closed-form Gaussian
// coverage of a blurred axis-aligned rectangle (Evan Wallace's technique).
fn erf(x: vec2<f32>) -> vec2<f32> {
    let s = sign(x);
    let a = abs(x);
    let x1 = 1.0 + (0.278393 + (0.230389 + (0.000972 + 0.078108 * a) * a) * a) * a;
    let x2 = x1 * x1;
    let x4 = x2 * x2;
    return s - s / x4;
}

fn gaussian_box_shadow(p: vec2<f32>, sigma: f32, half_size: vec2<f32>) -> f32 {
    let low = (p - half_size) / (sigma * 1.4142135);
    let high = (p + half_size) / (sigma * 1.4142135);
    let v = erf(high) - erf(low);
    return v.x * v.y * 0.25;
}

fn sd_round_rect(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) - r + min(max(q.x, q.y), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sigma = max(in.blur * 0.5, 0.001);

    var alpha: f32;
    if (in.radius <= 0.5) {
        alpha = gaussian_box_shadow(in.local_pos, sigma, in.half_size);
    } else {
        let d = sd_round_rect(in.local_pos, in.half_size, in.radius);
        let sharp = gaussian_box_shadow(in.local_pos, sigma, in.half_size);
        let rounded = 1.0 - smoothstep(-sigma, sigma, d);
        alpha = clamp(mix(sharp, rounded, 0.6), 0.0, 1.0);
    }

    if (in.inset > 0.5) {
        let box_d = sd_round_rect(in.box_local_pos, in.box_half_size, in.box_radius);
        let box_mask = 1.0 - smoothstep(-1.0, 1.0, box_d);
        alpha = (1.0 - alpha) * box_mask;
    }

    if (alpha <= 0.0) {
        discard;
    }

    var color = in.color;
    color.a = color.a * alpha;
    return color;
}