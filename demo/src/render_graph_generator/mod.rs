use rafx::api::{
    RafxFormat, RafxPrimitiveTopology, RafxResourceState, RafxResourceType, RafxResult,
    RafxSampleCount,
};
use rafx::framework::VertexDataSetLayout;
use rafx::framework::{ImageViewResource, ResourceArc};
use rafx::framework::{RenderResources, ResourceContext};
use rafx::graph::*;
use rafx::render_features::{ExtractResources, RenderView};

mod shadow_map_pass;
use shadow_map_pass::ShadowMapImageResources;

mod opaque_pass;
use opaque_pass::OpaquePass;

mod depth_prepass;

mod ssao_pass;

mod bloom_extract_pass;
use crate::demo_plugin::DemoStaticResources;
use crate::features::mesh::ShadowMapResource;
use crate::RenderOptions;
use bloom_extract_pass::BloomExtractPass;
use rafx::assets::AssetManager;
use rafx::renderer::RenderGraphGenerator;
use rafx::renderer::SwapchainResources;

mod bloom_blur_pass;

mod bloom_combine_pass;

mod ui_pass;

mod compute_test;

lazy_static::lazy_static! {
    pub static ref EMPTY_VERTEX_LAYOUT : VertexDataSetLayout = {
        VertexDataSetLayout::new(vec![], RafxPrimitiveTopology::TriangleList)
    };
}

// All the data that can influence the rendergraph
pub struct RenderGraphConfig {
    pub color_format: RafxFormat,
    pub depth_format: RafxFormat,
    pub swapchain_format: RafxFormat,
    pub samples: RafxSampleCount,
    pub enable_hdr: bool,
    pub enable_bloom: bool,
    pub show_surfaces: bool,
    pub blur_pass_count: usize,
}

// This just wraps a bunch of values so they don't have to be passed individually to all the passes
struct RenderGraphContext<'a> {
    graph: &'a mut RenderGraphBuilder,
    resource_context: &'a ResourceContext,
    graph_config: &'a RenderGraphConfig,
    main_view: &'a RenderView,
    extract_resources: &'a ExtractResources<'a>,
    render_resources: &'a RenderResources,
}

pub struct DemoRenderGraphGenerator;

impl RenderGraphGenerator for DemoRenderGraphGenerator {
    fn generate_render_graph(
        &self,
        asset_manager: &AssetManager,
        swapchain_image: ResourceArc<ImageViewResource>,
        main_view: RenderView,
        extract_resources: &ExtractResources,
        render_resources: &RenderResources,
    ) -> RafxResult<PreparedRenderGraph> {
        profiling::scope!("Build Render Graph");

        let device_context = asset_manager.device_context();
        let resource_context = asset_manager.resource_manager().resource_context();
        let swapchain_resources = render_resources.fetch::<SwapchainResources>();
        let static_resources = render_resources.fetch::<DemoStaticResources>();

        let graph_config = {
            let render_options = extract_resources.fetch::<RenderOptions>().clone();
            let swapchain_format = swapchain_resources.swapchain_surface_info.format;
            let sample_count = if render_options.enable_msaa {
                RafxSampleCount::SampleCount4
            } else {
                RafxSampleCount::SampleCount1
            };

            let color_format = if render_options.enable_hdr {
                swapchain_resources.default_color_format_hdr
            } else {
                swapchain_resources.default_color_format_sdr
            };

            RenderGraphConfig {
                color_format,
                depth_format: swapchain_resources.default_depth_format,
                samples: sample_count,
                enable_hdr: render_options.enable_hdr,
                swapchain_format,
                enable_bloom: render_options.enable_bloom,
                show_surfaces: render_options.show_surfaces,
                blur_pass_count: render_options.blur_pass_count,
            }
        };

        let mut graph = RenderGraphBuilder::default();

        let mut graph_context = RenderGraphContext {
            graph: &mut graph,
            resource_context: &resource_context,
            graph_config: &graph_config,
            main_view: &main_view,
            render_resources,
            extract_resources,
        };

        let depth_prepass = depth_prepass::depth_prepass(&mut graph_context);

        let ssao_material_pass = asset_manager
            .committed_asset(&static_resources.ssao_material)
            .unwrap()
            .get_single_material_pass()
            .unwrap();
        let ssao_pass =
            ssao_pass::ssao_pass(&mut graph_context, &depth_prepass, ssao_material_pass);

        let shadow_maps = shadow_map_pass::shadow_map_passes(&mut graph_context);

        let opaque_pass = opaque_pass::opaque_pass(
            &mut graph_context,
            depth_prepass.depth,
            ssao_pass.ambient_occlusion,
            &shadow_maps,
        );

        {
            let compute_test_pipeline = asset_manager
                .committed_asset(&static_resources.compute_test)
                .unwrap()
                .compute_pipeline
                .clone();

            let compute_test_pass =
                compute_test::compute_test_pass(&mut graph_context, &compute_test_pipeline);

            let _out = graph_context.graph.read_storage_buffer(
                opaque_pass.node,
                compute_test_pass.position_buffer,
                RenderGraphBufferConstraint {
                    ..Default::default()
                },
            );
        }

        let previous_pass_color = if graph_config.enable_hdr {
            let bloom_extract_material_pass = asset_manager
                .committed_asset(&static_resources.bloom_extract_material)
                .unwrap()
                .get_single_material_pass()
                .unwrap();

            let bloom_blur_material_pass = asset_manager
                .committed_asset(&static_resources.bloom_blur_material)
                .unwrap()
                .get_single_material_pass()
                .unwrap();

            let bloom_combine_material_pass = asset_manager
                .committed_asset(&static_resources.bloom_combine_material)
                .unwrap()
                .get_single_material_pass()
                .unwrap();

            let bloom_extract_pass = bloom_extract_pass::bloom_extract_pass(
                &mut graph_context,
                bloom_extract_material_pass,
                &opaque_pass,
            );

            let blurred_color = if graph_config.enable_bloom && graph_config.blur_pass_count > 0 {
                let bloom_blur_pass = bloom_blur_pass::bloom_blur_pass(
                    &mut graph_context,
                    bloom_blur_material_pass,
                    &bloom_extract_pass,
                );
                bloom_blur_pass.color
            } else {
                bloom_extract_pass.hdr_image
            };

            let bloom_combine_pass = bloom_combine_pass::bloom_combine_pass(
                &mut graph_context,
                bloom_combine_material_pass,
                &bloom_extract_pass,
                blurred_color,
            );

            bloom_combine_pass.color
        } else {
            opaque_pass.color
        };

        let ui_pass = ui_pass::ui_pass(&mut graph_context, previous_pass_color);

        let _swapchain_output_image_id = graph.set_output_image(
            ui_pass.color,
            swapchain_image,
            RenderGraphImageSpecification {
                samples: RafxSampleCount::SampleCount1,
                format: graph_config.swapchain_format,
                resource_type: RafxResourceType::TEXTURE | RafxResourceType::RENDER_TARGET_COLOR,
                extents: RenderGraphImageExtents::MatchSurface,
                layer_count: 1,
                mip_count: 1,
            },
            Default::default(),
            RafxResourceState::PRESENT,
        );

        let prepared_render_graph = PreparedRenderGraph::new(
            &device_context,
            &resource_context,
            graph,
            &swapchain_resources.swapchain_surface_info,
        )?;

        render_resources
            .fetch_mut::<ShadowMapResource>()
            .set_shadow_map_image_views(&prepared_render_graph);

        Ok(prepared_render_graph)
    }
}
