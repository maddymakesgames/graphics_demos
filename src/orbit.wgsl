struct OrbitUniform {
    a: u32;
    b: u32;
    timestep: u32;
    time: f32;
    color: vec3<f32>;
    show_line: u32;
    line_color: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> data: OrbitUniform;

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(1)]] pos: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.pos = model.position;
    return out;
}

fn in_range(val: f32, center: f32, range: f32) -> bool {
    return abs(val - center) <= range;
}

fn sqr(val: f32) -> f32 {
    return val * val;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var s: f32 = 100.0;

    if(sqr(in.pos.x - (f32(data.a) / s * cos(f32(data.time)))) + sqr(in.pos.y - f32(data.b) / s * sin(f32(data.time))) <= 0.001) {
        return vec4<f32>(data.color, 1.0);
    } else if(in_range(
        (in.pos.x * in.pos.x) / f32(data.a * data.a) + (in.pos.y * in.pos.y) / f32(data.b * data.b), 
        1.0 / (s * s), 
        0.000003) &&
        data.show_line == u32(1)
    ) {
        return vec4<f32>(data.line_color, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}