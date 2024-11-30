#import bevy_pbr::{
    mesh_view_bindings,
    mesh_bindings,
    mesh_bindings::mesh,
    mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_normal_local_to_world, get_world_from_local},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, calculate_view, prepare_world_normal, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::pbr_input_new,
    forward_io::FragmentOutput,
}

struct ChunkMaterial {
    reflectance: f32,
    perceptual_roughness: f32,
    metallic: f32,
}

@group(2) @binding(0) var<uniform> chunk_material: ChunkMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) vert_data: u32
};

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) ambient: f32,
    @location(2) world_pos: vec4<f32>,
    @location(3) blend_colour: vec3<f32>,
    @location(4) instance_index: u32,
}

var<private> normals: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
	vec3<f32>(-1.0, 0.0, 0.0), // Left
	vec3<f32>(1.0, 0.0, 0.0), // Right
	vec3<f32>(0.0, 0.0, 1.0), // Back
	vec3<f32>(0.0, 0.0, -1.0), // Front
	vec3<f32>(0.0, 1.0, 0.0), // Up
	vec3<f32>(0.0, -1.0, 0.0) // Down
);

var<private> ambient_lerps: vec4<f32> = vec4<f32>(1.0,0.7,0.5,0.15);

var<private> block_colour: array<vec3<f32>,2> = array<vec3<f32>,2>(
	vec3<f32>(0.0, 0.0, 0.0), // air
	vec3<f32>(5.0, 1.0, 3.0), // block
);

// var<private> regions: array<f32, 4> = array<f32, 4>(
//     -10., 0. , 10., 20
// );

// var<private> region_colours: array<vec3<f32>, 5> = array<vec3<f32>, 5>(
//     vec3<f32>(5.0,0.0,1.0),
//     vec3<f32>(1.0,0.2,5.0),
//     vec3<f32>(2.0,7.0,3.0),
//     vec3<f32>(0.0,5.0,4.0),
//     vec3<f32>(6.0,0.0,6.0)
// );

fn x_bits(bit_num: u32) -> u32 {
    return (1u << bit_num) - 1u;
}

@vertex 
fn vertex(vertex: Vertex) -> VertexOut {
    var out: VertexOut;

    // Unpack Vertex data into component parts
    let x = f32(vertex.vert_data & x_bits(6u));
    let y = f32((vertex.vert_data >> 6u) & x_bits(6u));
    let z = f32((vertex.vert_data >> 12u) & x_bits(6u));
    let ao = (vertex.vert_data >> 18u) & x_bits(3u);
    let normal_index = (vertex.vert_data >> 21u) & x_bits(3u);
    let block_index = (vertex.vert_data >> 24u) & x_bits(11u);

    let local_pos = vec4<f32>(x, y, z, 1.0); 
    let world_pos = get_world_from_local(vertex.instance_index) * local_pos;

    out.clip_pos = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        local_pos
    );
    out.world_normal = mesh_normal_local_to_world(normals[normal_index], vertex.instance_index);
    out.ambient = ambient_lerps[ao];
    out.world_pos = world_pos;

    let high = vec3<f32>(5.00, 0.2, 5.0);
    let low = vec3<f32>(1.0, 1.0, 9.0);
    let noise = (out.world_pos.y) / 32.;
    out.blend_colour = ((low * noise) + (high * (1.0-noise)));

    // if world_pos.y < regions[0] {
    //     out.blend_colour = region_colours[0];
    // } else if world_pos.y < regions[1] {
    //     out.blend_colour = region_colours[1];
    // } else if world_pos.y < regions[2] {
    //     out.blend_colour = region_colours[2];
    // } else if world_pos.y < regions[3] {
    //     out.blend_colour = region_colours[3];
    // } else {
    //     out.blend_colour = region_colours[4];
    // }
    
    // out.blend_colour = block_colour[block_index];
    out.instance_index = vertex.instance_index;

    return out;
}

@fragment
fn fragment(input: VertexOut) -> FragmentOutput {
    var pbr_input = pbr_input_new();

    pbr_input.flags = mesh[input.instance_index].flags;
    pbr_input.V = calculate_view(input.world_pos, false);
    pbr_input.frag_coord = input.clip_pos;
    pbr_input.world_position = input.world_pos;
    pbr_input.world_normal = prepare_world_normal(input.world_normal, false, false);

    pbr_input.material.base_color = vec4<f32>(input.blend_colour * input.ambient, 1.0);

    pbr_input.material.reflectance = chunk_material.reflectance;
    pbr_input.material.perceptual_roughness = chunk_material.perceptual_roughness;
    pbr_input.material.metallic = chunk_material.metallic;

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}
