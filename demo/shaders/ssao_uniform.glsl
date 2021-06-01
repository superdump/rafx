//
// Per-Frame Pass
//

// @[export]
// @[internal_buffer]
layout (set = 0, binding = 0) uniform PerViewData {
    mat4 proj;
    mat4 inv_proj;
} per_view_data;

// @[export]
// @[internal_buffer]
// @[slot_name("ssao_config")]
layout(set = 0, binding = 1) uniform SsaoConfigUbo {
    vec3 kernel[32];
    uint kernel_size;
    float radius_vs;
    float bias;
} ssao_config;

// @[immutable_samplers([
//     (
//         mag_filter: Linear,
//         min_filter: Linear,
//         mip_map_mode: Linear,
//         address_mode_u: ClampToEdge,
//         address_mode_v: ClampToEdge,
//         address_mode_w: ClampToEdge,
//     )
// ])]
layout (set = 0, binding = 2) uniform sampler clamp_sampler;

// @[immutable_samplers([
//     (
//         mag_filter: Linear,
//         min_filter: Linear,
//         mip_map_mode: Linear,
//         address_mode_u: Repeat,
//         address_mode_v: Repeat,
//         address_mode_w: Repeat,
//     )
// ])]
layout (set = 0, binding = 3) uniform sampler repeat_sampler;
