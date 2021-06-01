//
// Per-Frame Pass
//

// @[export]
// @[internal_buffer]
layout (set = 0, binding = 0) uniform PerViewData {
    mat4 view;
    mat4 view_proj;
    mat4 inv_view3;
} per_view_data;

// @[immutable_samplers([
//     (
//         mag_filter: Linear,
//         min_filter: Linear,
//         mip_map_mode: Linear,
//         address_mode_u: Repeat,
//         address_mode_v: Repeat,
//         address_mode_w: Repeat,
//         max_anisotropy: 16.0,
//     )
// ])]
layout (set = 0, binding = 1) uniform sampler smp;

// //
// // Per-Material Bindings
// //
// struct MaterialData {
//     float normal_texture_scale;
//     bool has_normal_texture;
// };

// // @[export]
// // @[internal_buffer]
// // @[slot_name("per_material_data")]
// layout (set = 1, binding = 0) uniform MaterialDataUbo {
//     MaterialData data;
// } per_material_data;

// @[export]
// @[internal_buffer]
layout(set = 2, binding = 0) uniform PerObjectData {
    mat4 model;
    mat4 inv_trans_model3;
} per_object_data;
