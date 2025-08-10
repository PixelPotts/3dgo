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
    let light_dir = normalize(vec3<f32>(0.8, 1.0, 0.6));
    let light_color = vec3<f32>(1.0, 0.95, 0.85);
    let ambient = vec3<f32>(0.15, 0.15, 0.2);
    
    // Add simple procedural texture based on world position
    let texture_scale = 8.0;
    let noise = sin(in.world_position.x * texture_scale) * 
                sin(in.world_position.y * texture_scale) * 
                sin(in.world_position.z * texture_scale);
    let texture_factor = 0.05 * noise + 0.95;
    
    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = diffuse_strength * light_color;
    
    let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    let reflect_dir = reflect(-light_dir, in.world_normal);
    let spec_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 64.0);
    let specular = spec_strength * light_color * 0.6;
    
    // Add rim lighting for better stone definition
    let rim_factor = 1.0 - max(dot(view_dir, in.world_normal), 0.0);
    let rim_light = pow(rim_factor, 2.0) * 0.2;
    
    // Calculate Z-depth darkening (stones further back appear darker)
    let view_space_z = (camera.view_proj * vec4<f32>(in.world_position, 1.0)).z;
    let depth_factor = 1.0 - (view_space_z * 0.015); // Subtle darkening based on depth
    let depth_factor_clamped = clamp(depth_factor, 0.7, 1.0); // Don't darken too much
    
    let final_color = (ambient + diffuse + specular + rim_light) * in.color * texture_factor * depth_factor_clamped;
    
    return vec4<f32>(final_color, 1.0);
}