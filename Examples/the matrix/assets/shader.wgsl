struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @location(1) color: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};


struct PushConstant {
    mvp: mat4x4<f32>,
};
var<push_constant> pc: PushConstant;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;

    result.position = pc.mvp * vec4(position, 0.0, 1.0);
    result.uv = uv;
    result.color = color;

    return result;
}


@group(0)
@binding(0)
var the_sampler: sampler;
@group(0)
@binding(1)
var color: texture_2d<f32>;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let v_color: vec4<f32> = textureSample(color, the_sampler, vertex.uv);

    return v_color.r * vec4(0.05, vertex.color.r, 0.2,  vertex.color.g);
}
