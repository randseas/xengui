// SPDX-License-Identifier: Apache-2.0
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radius: f32,
    @location(3) border_width: f32,
    @location(4) fill_color: vec4<f32>,
    @location(5) border_color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) border_width: f32,
    @location(5) fill_color: vec4<f32>,
    @location(6) border_color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.local_pos = local_pos;
    out.half_size = half_size;
    out.radius = radius;
    out.border_width = border_width;
    out.fill_color = fill_color;
    out.border_color = border_color;
    return out;
}

fn sd_round_rect(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) - r + min(max(q.x, q.y), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = sd_round_rect(in.local_pos, in.half_size, in.radius);
    let aa = max(fwidth(d) * 0.75, 0.0001);

    let outer_alpha = 1.0 - smoothstep(-aa, aa, d);
    if (outer_alpha <= 0.0) {
        discard;
    }

    var color = in.fill_color;
    if (in.border_width > 0.0) {
        let inner_d = d + in.border_width;
        let inner_mask = 1.0 - smoothstep(-aa, aa, inner_d);
        color = mix(in.border_color, in.fill_color, inner_mask);
    }

    color.a = color.a * outer_alpha;
    return color;
}