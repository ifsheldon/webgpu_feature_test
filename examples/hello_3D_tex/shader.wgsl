[[block]]
struct VertexUniforms{
    z:f32;
};

struct VertexOutput{
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coord: vec3<f32>;
};

[[group(0),binding(0)]] var<uniform> vertex_uniforms: VertexUniforms;

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> VertexOutput {
    var x = 0.0;
    var y = 0.0;
    let idx = i32(in_vertex_index);
    if (idx == 0){
        x = -1.0;
        y = -1.0;
    }elseif (idx == 1){
        x = 1.0;
        y = -1.0;
    }elseif (idx == 2){
        x = -1.0;
        y = 1.0;
    }else{
        x = 1.0;
        y = 1.0;
    }
    var out: VertexOutput;
    out.tex_coord = vec3<f32>((x+1.0)/2.0, (y+1.0)/2.0, vertex_uniforms.z);
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}


[[group(1),binding(0)]] var volume_tex: texture_3d<f32>;
[[group(1),binding(1)]] var volume_tex_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in : VertexOutput) -> [[location(0)]] vec4<f32> {
    let sampled_color = textureSample(volume_tex, volume_tex_sampler, in.tex_coord);
    return sampled_color;
}
