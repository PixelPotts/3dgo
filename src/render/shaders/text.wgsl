struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.tex_coords = tex_coords;
    return out;
}

@group(0) @binding(0)
var font_texture: texture_2d<f32>;
@group(0) @binding(1)
var font_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(font_texture, font_sampler, in.tex_coords).r;
    return vec4<f32>(1.0, 1.0, 1.0, alpha); // White text
}