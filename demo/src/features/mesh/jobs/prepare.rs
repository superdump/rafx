use rafx::render_feature_prepare_job_predule::*;

use super::*;
use crate::components::{
    DirectionalLightComponent, PointLightComponent, SpotLightComponent, TransformComponent,
};
use crate::phases::{
    DepthPrepassRenderPhase, OpaqueRenderPhase, ShadowMapRenderPhase, WireframeRenderPhase,
};
use rafx::base::resource_map::ReadBorrow;
use rafx::framework::{
    DescriptorSetAllocatorRef, MaterialPassResource, ResourceArc, ResourceContext,
};

use glam::{Mat3, Mat4};
use rafx::renderer::InvalidResources;
use shaders::depth_vert::PerObjectDataUniform as ShadowPerObjectShaderParam;
use shaders::depth_vert::PerViewDataUniform as ShadowPerViewShaderParam;
use shaders::mesh_textured_frag::PerObjectDataUniform as MeshPerObjectFragmentShaderParam;
use shaders::mesh_textured_frag::PerViewDataUniform as MeshPerViewFragmentShaderParam;

const PER_VIEW_DESCRIPTOR_SET_INDEX: u32 =
    shaders::mesh_textured_frag::PER_VIEW_DATA_DESCRIPTOR_SET_INDEX as u32;
const PER_MATERIAL_DESCRIPTOR_SET_INDEX: u32 =
    shaders::mesh_textured_frag::PER_MATERIAL_DATA_DESCRIPTOR_SET_INDEX as u32;
const PER_INSTANCE_DESCRIPTOR_SET_INDEX: u32 =
    shaders::mesh_textured_frag::PER_OBJECT_DATA_DESCRIPTOR_SET_INDEX as u32;

struct PreparedDirectionalLight<'a> {
    light: &'a DirectionalLightComponent,
    shadow_map_index: Option<usize>,
}

struct PreparedPointLight<'a> {
    light: &'a PointLightComponent,
    transform: &'a TransformComponent,
    shadow_map_index: Option<usize>,
}

struct PreparedSpotLight<'a> {
    light: &'a SpotLightComponent,
    transform: &'a TransformComponent,
    shadow_map_index: Option<usize>,
}

pub struct MeshPrepareJob<'prepare> {
    resource_context: ResourceContext,
    requires_textured_descriptor_sets: bool,
    requires_untextured_descriptor_sets: bool,
    depth_material_pass: Option<ResourceArc<MaterialPassResource>>,
    shadow_map_data: ReadBorrow<'prepare, ShadowMapResource>,
    invalid_resources: ReadBorrow<'prepare, InvalidResources>,
    mesh_part_descriptor_sets: Arc<AtomicOnceCellStack<MeshPartDescriptorSetPair>>,
    render_objects: MeshRenderObjectSet,
}

impl<'prepare> MeshPrepareJob<'prepare> {
    pub fn new(
        prepare_context: &RenderJobPrepareContext<'prepare>,
        frame_packet: Box<MeshFramePacket>,
        submit_packet: Box<MeshSubmitPacket>,
        render_objects: MeshRenderObjectSet,
        max_num_mesh_parts: Option<usize>,
    ) -> Arc<dyn RenderFeaturePrepareJob<'prepare> + 'prepare> {
        let max_num_mesh_part_descriptor_sets = if let Some(max_num_mesh_parts) = max_num_mesh_parts
        {
            frame_packet.render_object_instances().len() * max_num_mesh_parts
        } else {
            // NOTE(dvd): Count exact number of mesh parts required.
            let mut num_mesh_part_descriptor_sets = 0;
            for id in 0..frame_packet.render_object_instances().len() {
                // TODO(dvd): This could be replaced by an `iter` or `as_slice` method on the data.
                if let Some(extracted_data) = frame_packet.render_object_instances_data().get(id) {
                    let mesh_parts = &extracted_data.mesh_asset.inner.mesh_parts;
                    num_mesh_part_descriptor_sets += mesh_parts
                        .iter()
                        .filter(|mesh_part| mesh_part.is_some())
                        .count();
                }
            }
            num_mesh_part_descriptor_sets
        };

        let mut requires_textured_descriptor_sets = false;
        let mut requires_untextured_descriptor_sets = false;

        for view in frame_packet.view_packets() {
            if view
                .view()
                .feature_flag_is_relevant::<MeshUntexturedRenderFeatureFlag>()
            {
                requires_untextured_descriptor_sets = true;
            } else {
                requires_textured_descriptor_sets = true;
            }
        }

        Arc::new(PrepareJob::new(
            Self {
                resource_context: { prepare_context.resource_context.clone() },
                mesh_part_descriptor_sets: {
                    // TODO: Ideally this would use an allocator from the `prepare_context`.
                    Arc::new(AtomicOnceCellStack::with_capacity(
                        max_num_mesh_part_descriptor_sets,
                    ))
                },
                depth_material_pass: {
                    frame_packet
                        .per_frame_data()
                        .get()
                        .depth_material_pass
                        .clone()
                },
                shadow_map_data: {
                    prepare_context
                        .render_resources
                        .fetch::<ShadowMapResource>()
                },
                requires_textured_descriptor_sets,
                requires_untextured_descriptor_sets,
                invalid_resources: { prepare_context.render_resources.fetch::<InvalidResources>() },
                render_objects,
            },
            frame_packet,
            submit_packet,
        ))
    }
}

pub struct MeshPrepareJobContext {
    descriptor_set_allocator: DescriptorSetAllocatorRef,
}

impl<'prepare> PrepareJobEntryPoints<'prepare> for MeshPrepareJob<'prepare> {
    fn begin_per_frame_prepare(
        &self,
        context: &PreparePerFrameContext<'prepare, '_, Self>,
    ) {
        let mut per_frame_submit_data = Box::new(MeshPerFrameSubmitData {
            num_shadow_map_2d: 0,
            shadow_map_2d_data: Default::default(),
            shadow_map_2d_image_views: Default::default(),
            num_shadow_map_cube: 0,
            shadow_map_cube_data: Default::default(),
            shadow_map_cube_image_views: Default::default(),
            shadow_map_image_index_remap: [None; MAX_SHADOW_MAPS_CUBE + MAX_SHADOW_MAPS_2D],
            mesh_part_descriptor_sets: self.mesh_part_descriptor_sets.clone(),
        });

        let shadow_map_data = &self.shadow_map_data;
        let invalid_resources = &self.invalid_resources;

        //
        // Create uniform data for each shadow map and a properly-sized static array of the images.
        // This will take our mixed list of shadow maps and separate them into 2d (spot and
        // directional lights) and cube (point lights)
        //

        // This maps the index in the combined list to indices in the 2d/cube maps

        assert_eq!(
            shadow_map_data.shadow_map_render_views.len(),
            shadow_map_data.shadow_map_image_views.len()
        );

        {
            profiling::scope!("gather shadow data");

            for (index, shadow_map_render_view) in
                shadow_map_data.shadow_map_render_views.iter().enumerate()
            {
                match shadow_map_render_view {
                    ShadowMapRenderView::Single(shadow_view) => {
                        let num_shadow_map_2d = per_frame_submit_data.num_shadow_map_2d;
                        if num_shadow_map_2d >= MAX_SHADOW_MAPS_2D {
                            log::warn!("More 2D shadow maps than the mesh shader can support");
                            continue;
                        }

                        per_frame_submit_data.shadow_map_2d_data[num_shadow_map_2d] =
                            shaders::mesh_textured_frag::ShadowMap2DDataStd140 {
                                shadow_map_view_proj: shadow_view.view_proj().to_cols_array_2d(),
                                shadow_map_light_dir: shadow_view.view_dir().into(),
                                ..Default::default()
                            };

                        per_frame_submit_data.shadow_map_2d_image_views[num_shadow_map_2d] =
                            Some(shadow_map_data.shadow_map_image_views[index].clone());
                        per_frame_submit_data.shadow_map_image_index_remap[index] =
                            Some(num_shadow_map_2d);

                        per_frame_submit_data.num_shadow_map_2d += 1;
                    }
                    ShadowMapRenderView::Cube(shadow_views) => {
                        let num_shadow_map_cube = per_frame_submit_data.num_shadow_map_cube;
                        if num_shadow_map_cube >= MAX_SHADOW_MAPS_CUBE {
                            log::warn!("More cube shadow maps than the mesh shader can support");
                            continue;
                        }

                        // Shader not set up for infinite far plane
                        let (near, far) = shadow_views[0]
                            .depth_range()
                            .finite_planes_after_reverse()
                            .unwrap();

                        per_frame_submit_data.shadow_map_cube_data[num_shadow_map_cube] =
                            shaders::mesh_textured_frag::ShadowMapCubeDataStd140 {
                                cube_map_projection_near_z: near,
                                cube_map_projection_far_z: far,
                                ..Default::default()
                            };

                        per_frame_submit_data.shadow_map_cube_image_views[num_shadow_map_cube] =
                            Some(shadow_map_data.shadow_map_image_views[index].clone());
                        per_frame_submit_data.shadow_map_image_index_remap[index] =
                            Some(num_shadow_map_cube);
                        per_frame_submit_data.num_shadow_map_cube += 1;
                    }
                }
            }

            for index in per_frame_submit_data.num_shadow_map_2d..MAX_SHADOW_MAPS_2D {
                per_frame_submit_data.shadow_map_2d_image_views[index] =
                    Some(invalid_resources.invalid_image_depth.clone());
            }

            for index in per_frame_submit_data.num_shadow_map_cube..MAX_SHADOW_MAPS_CUBE {
                per_frame_submit_data.shadow_map_cube_image_views[index] =
                    Some(invalid_resources.invalid_cube_map_image_depth.clone());
            }
        }

        context
            .submit_packet()
            .per_frame_submit_data()
            .set(per_frame_submit_data);
    }

    fn prepare_render_object_instance(
        &self,
        job_context: &mut MeshPrepareJobContext,
        context: &PrepareRenderObjectInstanceContext<'prepare, '_, Self>,
    ) {
        let render_object_instance = context.render_object_instance_data();
        if render_object_instance.is_none() {
            return;
        }

        let extracted_data = render_object_instance.as_ref().unwrap();
        let world_transform = Mat4::from_scale_rotation_translation(
            extracted_data.scale,
            extracted_data.rotation,
            extracted_data.translation,
        );

        let num_mesh_parts = extracted_data.mesh_asset.inner.mesh_parts.len();
        let start_index = self
            .mesh_part_descriptor_sets
            .reserve_uninit(num_mesh_parts);

        context.set_render_object_instance_submit_data(MeshRenderObjectInstanceSubmitData {
            mesh_part_descriptor_set_index: start_index,
        });

        let model = world_transform.to_cols_array_2d();
        let inv_trans_model3 = {
            let model3 = Mat3::from_cols_array_2d(&[
                [model[0][0], model[0][1], model[0][2]],
                [model[1][0], model[1][1], model[1][2]],
                [model[2][0], model[2][1], model[2][2]],
            ]);
            let inv_trans_model3 = model3.inverse().transpose().to_cols_array_2d();
            [
                [
                    inv_trans_model3[0][0],
                    inv_trans_model3[0][1],
                    inv_trans_model3[0][2],
                    0.0,
                ],
                [
                    inv_trans_model3[1][0],
                    inv_trans_model3[1][1],
                    inv_trans_model3[1][2],
                    0.0,
                ],
                [
                    inv_trans_model3[2][0],
                    inv_trans_model3[2][1],
                    inv_trans_model3[2][2],
                    0.0,
                ],
                [0.0, 0.0, 0.0, 1.0],
            ]
        };
        let descriptor_set_allocator = &mut job_context.descriptor_set_allocator;
        let depth_descriptor_set = {
            let per_object_data = ShadowPerObjectShaderParam {
                model,
                inv_trans_model3,
            };

            let per_instance_descriptor_set_layout = &self
                .depth_material_pass
                .as_ref()
                .unwrap()
                .get_raw()
                .descriptor_set_layouts[PER_INSTANCE_DESCRIPTOR_SET_INDEX as usize];

            descriptor_set_allocator
                .create_descriptor_set_with_writer(
                    per_instance_descriptor_set_layout,
                    shaders::depth_vert::DescriptorSet2Args {
                        per_object_data: &per_object_data,
                    },
                )
                .unwrap()
        };

        for (mesh_part_index, mesh_part) in extracted_data
            .mesh_asset
            .inner
            .mesh_parts
            .iter()
            .enumerate()
        {
            if mesh_part.is_none() {
                continue;
            }

            let mesh_part = mesh_part.as_ref().unwrap();
            let textured_descriptor_set = if self.requires_textured_descriptor_sets {
                let per_object_data = MeshPerObjectFragmentShaderParam { model };

                let per_instance_descriptor_set_layout =
                    &mesh_part.material_instance.material.passes[mesh_part.textured_pass_index]
                        .material_pass_resource
                        .get_raw()
                        .descriptor_set_layouts
                        [PER_INSTANCE_DESCRIPTOR_SET_INDEX as usize];

                descriptor_set_allocator
                    .create_descriptor_set_with_writer(
                        per_instance_descriptor_set_layout,
                        shaders::mesh_textured_frag::DescriptorSet2Args {
                            per_object_data: &per_object_data,
                        },
                    )
                    .ok()
            } else {
                None
            };

            let untextured_descriptor_set = if self.requires_untextured_descriptor_sets {
                let per_object_data = MeshPerObjectFragmentShaderParam { model };

                let per_instance_descriptor_set_layout =
                    &mesh_part.material_instance.material.passes[mesh_part.untextured_pass_index]
                        .material_pass_resource
                        .get_raw()
                        .descriptor_set_layouts
                        [PER_INSTANCE_DESCRIPTOR_SET_INDEX as usize];

                descriptor_set_allocator
                    .create_descriptor_set_with_writer(
                        per_instance_descriptor_set_layout,
                        shaders::mesh_textured_frag::DescriptorSet2Args {
                            per_object_data: &per_object_data,
                        },
                    )
                    .ok()
            } else {
                None
            };

            self.mesh_part_descriptor_sets.set(
                start_index + mesh_part_index,
                MeshPartDescriptorSetPair {
                    depth_descriptor_set: depth_descriptor_set.clone(),
                    textured_descriptor_set,
                    untextured_descriptor_set,
                },
            );
        }
    }

    fn prepare_render_object_instance_per_view(
        &self,
        _job_context: &mut DefaultJobContext,
        context: &PrepareRenderObjectInstancePerViewContext<'prepare, '_, Self>,
    ) {
        let view = context.view();
        if let Some(extracted_data) = context.render_object_instance_data() {
            let distance = (view.eye_position() - extracted_data.translation).length_squared();
            let mesh_asset = &extracted_data.mesh_asset;
            let mesh_part_descriptor_set_index = context
                .render_object_instance_submit_data()
                .mesh_part_descriptor_set_index;

            for (mesh_part_index, mesh_part) in extracted_data
                .mesh_asset
                .inner
                .mesh_parts
                .iter()
                .enumerate()
            {
                if mesh_part.is_none() {
                    continue;
                }

                if view.phase_is_relevant::<DepthPrepassRenderPhase>() {
                    context.push_submit_node::<DepthPrepassRenderPhase>(
                        MeshDrawCall {
                            mesh_asset: mesh_asset.clone(),
                            mesh_part_index,
                            mesh_part_descriptor_set_index,
                        },
                        0,
                        distance,
                    );
                }

                if view.phase_is_relevant::<OpaqueRenderPhase>() {
                    context.push_submit_node::<OpaqueRenderPhase>(
                        MeshDrawCall {
                            mesh_asset: mesh_asset.clone(),
                            mesh_part_index,
                            mesh_part_descriptor_set_index,
                        },
                        0,
                        distance,
                    );
                }

                if view.phase_is_relevant::<ShadowMapRenderPhase>() {
                    context.push_submit_node::<ShadowMapRenderPhase>(
                        MeshDrawCall {
                            mesh_asset: mesh_asset.clone(),
                            mesh_part_index,
                            mesh_part_descriptor_set_index,
                        },
                        0,
                        distance,
                    );
                }

                if view.phase_is_relevant::<WireframeRenderPhase>()
                    && view.feature_flag_is_relevant::<MeshWireframeRenderFeatureFlag>()
                {
                    context.push_submit_node::<WireframeRenderPhase>(
                        MeshDrawCall {
                            mesh_asset: mesh_asset.clone(),
                            mesh_part_index,
                            mesh_part_descriptor_set_index,
                        },
                        0,
                        distance,
                    );
                }
            }
        }
    }

    fn end_per_view_prepare(
        &self,
        context: &PreparePerViewContext<'prepare, '_, Self>,
    ) {
        let mut descriptor_set_allocator = self.resource_context.create_descriptor_set_allocator();
        let shadow_map_data = &self.shadow_map_data;

        let per_view_data = context.per_view_data();
        let per_frame_submit_data = context.per_frame_submit_data();

        let view = context.view();
        let is_lit = !view.feature_flag_is_relevant::<MeshUnlitRenderFeatureFlag>();
        let has_shadows = !view.feature_flag_is_relevant::<MeshNoShadowsRenderFeatureFlag>();

        let opaque_descriptor_set = if view.phase_is_relevant::<OpaqueRenderPhase>() {
            let per_view_frag_data = {
                let mut per_view_frag_data = MeshPerViewFragmentShaderParam::default();

                per_view_frag_data.view = view.view_matrix().to_cols_array_2d();
                per_view_frag_data.view_proj = view.view_proj().to_cols_array_2d();
                per_view_frag_data.ambient_light = if is_lit {
                    glam::Vec4::new(0.03, 0.03, 0.03, 1.0).into()
                } else {
                    glam::Vec4::ONE.into()
                };

                for directional_light in &per_view_data.directional_lights {
                    if directional_light.is_none() {
                        break;
                    }

                    let light_count = per_view_frag_data.directional_light_count as usize;
                    if light_count > per_view_frag_data.directional_lights.len() {
                        break;
                    }

                    let directional_light = directional_light.as_ref().unwrap();
                    let light = directional_light;
                    let shadow_map_index = shadow_map_data
                        .shadow_map_lookup
                        .get(&LightId::DirectionalLight(directional_light.object_id))
                        .map(|x| per_frame_submit_data.shadow_map_image_index_remap[*x])
                        .flatten();

                    let light_from = glam::Vec3::ZERO;
                    let light_from_vs = (view.view_matrix() * light_from.extend(1.0)).truncate();
                    let light_to = light.light.direction;
                    let light_to_vs = (view.view_matrix() * light_to.extend(1.0)).truncate();

                    let light_direction = (light_to - light_from).normalize();
                    let light_direction_vs = (light_to_vs - light_from_vs).normalize();

                    let out = &mut per_view_frag_data.directional_lights[light_count];
                    out.direction_ws = light_direction.into();
                    out.direction_vs = light_direction_vs.into();
                    out.color = light.light.color.into();
                    out.intensity = light.light.intensity;
                    out.shadow_map = if has_shadows {
                        shadow_map_index.map(|x| x as i32).unwrap_or(-1)
                    } else {
                        -1
                    };

                    per_view_frag_data.directional_light_count += 1;
                }

                for point_light in &per_view_data.point_lights {
                    if point_light.is_none() {
                        break;
                    }

                    let light_count = per_view_frag_data.point_light_count as usize;
                    if light_count > per_view_frag_data.point_lights.len() {
                        break;
                    }

                    let point_light = point_light.as_ref().unwrap();
                    let light = point_light;
                    let shadow_map_index = shadow_map_data
                        .shadow_map_lookup
                        .get(&LightId::PointLight(point_light.object_id))
                        .map(|x| per_frame_submit_data.shadow_map_image_index_remap[*x])
                        .flatten();

                    let out = &mut per_view_frag_data.point_lights[light_count];
                    out.position_ws = light.transform.translation.into();
                    out.position_vs = (view.view_matrix()
                        * light.transform.translation.extend(1.0))
                    .truncate()
                    .into();
                    out.color = light.light.color.into();
                    out.range = light.light.range;
                    out.intensity = light.light.intensity;
                    out.shadow_map = if has_shadows {
                        shadow_map_index.map(|x| x as i32).unwrap_or(-1)
                    } else {
                        -1
                    };

                    per_view_frag_data.point_light_count += 1;
                }

                for spot_light in &per_view_data.spot_lights {
                    if spot_light.is_none() {
                        break;
                    }

                    let light_count = per_view_frag_data.spot_light_count as usize;
                    if light_count > per_view_frag_data.spot_lights.len() {
                        break;
                    }

                    let spot_light = spot_light.as_ref().unwrap();
                    let light = spot_light;
                    let shadow_map_index = shadow_map_data
                        .shadow_map_lookup
                        .get(&LightId::SpotLight(spot_light.object_id))
                        .map(|x| per_frame_submit_data.shadow_map_image_index_remap[*x])
                        .flatten();

                    let light_from = light.transform.translation;
                    let light_from_vs = (view.view_matrix() * light_from.extend(1.0)).truncate();
                    let light_to = light.transform.translation + light.light.direction;
                    let light_to_vs = (view.view_matrix() * light_to.extend(1.0)).truncate();

                    let light_direction = (light_to - light_from).normalize();
                    let light_direction_vs = (light_to_vs - light_from_vs).normalize();

                    let out = &mut per_view_frag_data.spot_lights[light_count];
                    out.position_ws = light_from.into();
                    out.position_vs = light_from_vs.into();
                    out.direction_ws = light_direction.into();
                    out.direction_vs = light_direction_vs.into();
                    out.spotlight_half_angle = light.light.spotlight_half_angle;
                    out.color = light.light.color.into();
                    out.range = light.light.range;
                    out.intensity = light.light.intensity;
                    out.shadow_map = if has_shadows {
                        shadow_map_index.map(|x| x as i32).unwrap_or(-1)
                    } else {
                        -1
                    };

                    per_view_frag_data.spot_light_count += 1;
                }

                per_view_frag_data.shadow_map_2d_data = per_frame_submit_data.shadow_map_2d_data;
                per_view_frag_data.shadow_map_cube_data =
                    per_frame_submit_data.shadow_map_cube_data;

                per_view_frag_data
            };

            let shadow_map_images = &mut [None; MAX_SHADOW_MAPS_2D];
            for index in 0..MAX_SHADOW_MAPS_2D {
                let image_view = per_frame_submit_data.shadow_map_2d_image_views[index]
                    .as_ref()
                    .unwrap();
                shadow_map_images[index] = Some(image_view);
            }

            let shadow_map_images_cube = &mut [None; MAX_SHADOW_MAPS_CUBE];
            for index in 0..MAX_SHADOW_MAPS_CUBE {
                let image_view = per_frame_submit_data.shadow_map_cube_image_views[index]
                    .as_ref()
                    .unwrap();
                shadow_map_images_cube[index] = Some(image_view);
            }

            // NOTE(dvd): This assumes that all opaque materials have the same per view descriptor set layout.
            let opaque_per_view_descriptor_set_layout = {
                let mut opaque_per_view_descriptor_set_layout = None;

                for id in 0..context.render_object_instances().len() {
                    // TODO(dvd): This could be replaced by an `iter` or `as_slice` method on the data.
                    if let Some(extracted_data) = context.render_object_instances_data().get(id) {
                        let mesh_parts = &extracted_data.mesh_asset.inner.mesh_parts;
                        for mesh_part in mesh_parts {
                            if let Some(mesh_part) = mesh_part {
                                opaque_per_view_descriptor_set_layout = Some(
                                    mesh_part
                                        .get_material_pass_resource(
                                            view,
                                            OpaqueRenderPhase::render_phase_index(),
                                        )
                                        .get_raw()
                                        .descriptor_set_layouts
                                        [PER_VIEW_DESCRIPTOR_SET_INDEX as usize]
                                        .clone(),
                                );
                                break;
                            }
                        }
                    }

                    if opaque_per_view_descriptor_set_layout.is_some() {
                        break;
                    }
                }

                opaque_per_view_descriptor_set_layout
            };

            opaque_per_view_descriptor_set_layout.as_ref().and_then(
                |per_view_descriptor_set_layout| {
                    descriptor_set_allocator
                        .create_descriptor_set(
                            &per_view_descriptor_set_layout,
                            shaders::mesh_textured_frag::DescriptorSet0Args {
                                shadow_map_images,
                                shadow_map_images_cube,
                                per_view_data: &per_view_frag_data,
                            },
                        )
                        .ok()
                },
            )
        } else {
            None
        };

        let depth_descriptor_set = {
            let mut per_view_data = ShadowPerViewShaderParam::default();

            per_view_data.view = view.view_matrix().to_cols_array_2d();
            per_view_data.view_proj = view.view_proj().to_cols_array_2d();
            per_view_data.inv_view3 = {
                let view_temp = view.view_matrix().to_cols_array_2d();
                let inv_view3 = Mat3::from_cols_array_2d(&[
                    [view_temp[0][0], view_temp[0][1], view_temp[0][2]],
                    [view_temp[1][0], view_temp[1][1], view_temp[1][2]],
                    [view_temp[2][0], view_temp[2][1], view_temp[2][2]],
                ])
                .inverse()
                .to_cols_array_2d();
                [
                    [inv_view3[0][0], inv_view3[0][1], inv_view3[0][2], 0.0],
                    [inv_view3[1][0], inv_view3[1][1], inv_view3[1][2], 0.0],
                    [inv_view3[2][0], inv_view3[2][1], inv_view3[2][2], 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ]
            };
            let per_instance_descriptor_set_layout = &self
                .depth_material_pass
                .as_ref()
                .unwrap()
                .get_raw()
                .descriptor_set_layouts[PER_VIEW_DESCRIPTOR_SET_INDEX as usize];

            descriptor_set_allocator
                .create_descriptor_set(
                    per_instance_descriptor_set_layout,
                    shaders::depth_vert::DescriptorSet0Args {
                        per_view_data: &per_view_data,
                    },
                )
                .ok()
        };

        context
            .view_submit_packet()
            .per_view_submit_data()
            .set(MeshPerViewSubmitData {
                opaque_descriptor_set,
                depth_descriptor_set,
            });
    }

    fn feature_debug_constants(&self) -> &'static RenderFeatureDebugConstants {
        super::render_feature_debug_constants()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        super::render_feature_index()
    }

    fn new_render_object_instance_job_context(&'prepare self) -> Option<MeshPrepareJobContext> {
        Some(MeshPrepareJobContext {
            descriptor_set_allocator: self.resource_context.create_descriptor_set_allocator(),
        })
    }

    fn new_render_object_instance_per_view_job_context(
        &'prepare self
    ) -> Option<DefaultJobContext> {
        Some(DefaultJobContext::new())
    }

    type RenderObjectInstanceJobContextT = MeshPrepareJobContext;
    type RenderObjectInstancePerViewJobContextT = DefaultJobContext;

    type FramePacketDataT = MeshRenderFeatureTypes;
    type SubmitPacketDataT = MeshRenderFeatureTypes;
}
