struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) color: vec3<f32>,
}

struct InstanceInput {
    @location(4) model_matrix_0: vec4<f32>,
    @location(5) model_matrix_1: vec4<f32>,
    @location(6) model_matrix_2: vec4<f32>,
    @location(7) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) color: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    let world_normal = normalize((model_matrix * vec4<f32>(model.normal, 0.0)).xyz);
    
    var out: VertexOutput;
    out.world_position = world_position.xyz;
    out.world_normal = world_normal;
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    out.clip_position = camera.view_proj * world_position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let light_color = vec3<f32>(0.8, 0.9, 1.0);
    let ambient = vec3<f32>(0.1, 0.15, 0.2);
    
    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = diffuse_strength * light_color * 0.3;
    
    let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    let reflect_dir = reflect(-light_dir, in.world_normal);
    let spec_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 16.0);
    let specular = spec_strength * light_color * 0.1;
    
    let final_color = (ambient + diffuse + specular) * in.color;
    
    // Increased opacity for better visibility (~12% = 4% + 8%)
    return vec4<f32>(final_color, 0.12);
}