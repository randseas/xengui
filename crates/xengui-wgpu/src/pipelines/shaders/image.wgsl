// SPDX-License-Identifier: Apache-2.0
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) local_pos: vec2<f32>,
    @location(3) half_size: vec2<f32>,
    @location(4) radius: f32,
    @location(5) tint: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) tint: vec4<f32>,
};

@group(0) @binding(0) var t_image: texture_2d<f32>;
@group(0) @binding(1) var s_image: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.local_pos = in.local_pos;
    out.half_size = in.half_size;
    out.radius = in.radius;
    out.tint = in.tint;
    return out;
}

fn rounded_rect_sdf(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - (half_size - vec2<f32>(radius, radius));
    return length(max(q, vec2<f32>(0.0, 0.0))) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_image, s_image, in.uv) * in.tint;

    if (in.radius > 0.0) {
        let dist = rounded_rect_sdf(in.local_pos, in.half_size, in.radius);
        let alpha_mask = 1.0 - smoothstep(-1.0, 1.0, dist);
        color.a = color.a * alpha_mask;
    }

    return color;
}