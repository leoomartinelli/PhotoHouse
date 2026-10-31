@group(0) @binding(0) var mask: texture_2d<f32>;
@group(0) @binding(1) var mask_sampler: sampler;
struct Vertex { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs(@builtin(vertex_index) i: u32) -> Vertex {
    let points = array<vec2<f32>, 6>(vec2(0.,0.), vec2(1.,0.), vec2(0.,1.), vec2(0.,1.), vec2(1.,0.), vec2(1.,1.));
    let p = points[i];
    return Vertex(vec4(p.x * 2. - 1., 1. - p.y * 2., 0., 1.), p);
}
@fragment fn fs(v: Vertex) -> @location(0) vec4<f32> {
    let coverage = textureSample(mask, mask_sampler, v.uv).a;
    return vec4(coverage);
}
