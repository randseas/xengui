// SPDX-License-Identifier: Apache-2.0
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radius: f32,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) radius: f32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.local_pos = local_pos;
    out.half_size = half_size;
    out.radius = radius;
    return out;
}

fn sd_round_rect(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) - r + min(max(q.x, q.y), 0.0);
}

// Multiplies existing framebuffer content by (1 - outside), erasing the
// window's four corners via the pipeline's blend state instead of writing
// new color - see WindowMaskPipeline for the matching BlendState.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = sd_round_rect(in.local_pos, in.half_size, in.radius);
    let aa = max(fwidth(d) * 0.75, 0.0001);
    let outside = smoothstep(-aa, aa, d);
    return vec4<f32>(0.0, 0.0, 0.0, outside);
}