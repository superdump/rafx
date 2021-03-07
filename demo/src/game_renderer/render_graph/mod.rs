use rafx::api::{
    RafxDeviceContext, RafxFormat, RafxPrimitiveTopology, RafxResourceState, RafxResourceType,
    RafxResult, RafxSampleCount,
};
use rafx::framework::{ResourceContext, RenderResources};
use rafx::framework::VertexDataSetLayout;
use rafx::framework::{ImageViewResource, ResourceArc};
use rafx::graph::*;
use rafx::nodes::{RenderJobBeginExecuteGraphContext, RenderView, ExtractResources};

mod shadow_map_pass;
use shadow_map_pass::ShadowMapImageResources;

mod opaque_pass;
use opaque_pass::OpaquePass;

mod bloom_extract_pass;
use crate::game_renderer::swapchain_resources::SwapchainResources;
use crate::game_renderer::GameRendererStaticResources;
use bloom_extract_pass::BloomExtractPass;
use rafx::assets::AssetManager;
use crate::RenderOptions;
use crate::features::mesh::shadow_map_resource::ShadowMapResource;

mod bloom_blur_pass;

mod bloom_combine_pass;

mod ui_pass;

mod compute_test;

lazy_static::lazy_static! {
    pub static ref EMPTY_VERTEX_LAYOUT : VertexDataSetLayout = {
        VertexDataSetLayout::new(vec![], RafxPrimitiveTopology::TriangleList)
    };
}

// Everything produced by the graph. This includes resources that may be needed during the prepare
// phase
pub struct BuildRenderGraphResult {
    pub executor: RenderGraphExecutor,
}

// All the data that can influence the rendergraph
pub struct RenderGraphConfig {
    pub color_format: RafxFormat,
    pub depth_format: RafxFormat,
    pub swapchain_format: RafxFormat,
    pub samples: RafxSampleCount,
    pub enable_hdr: bool,
    pub enable_bloom: bool,
    pub blur_pass_count: usize,
}

// This just wraps a bunch of values so they don't have to be passed individually to all the passes
struct RenderGraphContext<'a> {
    graph: &'a mut RenderGraphBuilder,
    resource_context: &'a ResourceContext,
    graph_config: &'a RenderGraphConfig,
    graph_callbacks: &'a mut RenderGraphNodeCallbacks,
    main_view: &'a RenderView,
    extract_resources: &'a ExtractResources<'a>,
    render_resources: &'a RenderResources,
}

pub fn build_render_graph(
    device_context: &RafxDeviceContext,
    resource_context: &ResourceContext,
    asset_manager: &AssetManager,
    swapchain_image: ResourceArc<ImageViewResource>,
    main_view: RenderView,
    swapchain_resources: &SwapchainResources,
    static_resources: &GameRendererStaticResources,
    extract_resources: &ExtractResources,
    render_resources: &RenderResources,
) -> RafxResult<BuildRenderGraphResult> {
    profiling::scope!("Build Render Graph");

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
            blur_pass_count: render_options.blur_pass_count,
        }
    };

    let mut graph = RenderGraphBuilder::default();
    let mut graph_callbacks = RenderGraphNodeCallbacks::default();

    let mut graph_context = RenderGraphContext {
        graph: &mut graph,
        resource_context,
        graph_callbacks: &mut graph_callbacks,
        graph_config: &graph_config,
        main_view: &main_view,
        render_resources,
        extract_resources,
    };

    let shadow_maps = shadow_map_pass::shadow_map_passes(&mut graph_context);

    let compute_test_pipeline = asset_manager
        .committed_asset(&static_resources.compute_test)
        .unwrap()
        .compute_pipeline
        .clone();

    let compute_test_pass =
        compute_test::compute_test_pass(&mut graph_context, &compute_test_pipeline);

    let bloom_extract_material_pass = asset_manager
        .get_material_pass_by_index(&static_resources.bloom_extract_material, 0)
        .unwrap();

    let bloom_blur_material_pass = asset_manager
        .get_material_pass_by_index(&static_resources.bloom_blur_material, 0)
        .unwrap();

    let bloom_combine_material_pass = asset_manager
        .get_material_pass_by_index(&static_resources.bloom_combine_material, 0)
        .unwrap();

    let skybox_material_pass = asset_manager
        .get_material_pass_by_index(&static_resources.skybox_material, 0)
        .unwrap();

    let skybox_texture = asset_manager
        .committed_asset(&static_resources.skybox_texture)
        .unwrap()
        .image_view
        .clone();

    let opaque_pass = opaque_pass::opaque_pass(
        &mut graph_context,
        skybox_material_pass,
        skybox_texture,
        &shadow_maps,
    );

    {
        let _out = graph_context.graph.read_storage_buffer(
            opaque_pass.node,
            compute_test_pass.position_buffer,
            RenderGraphBufferConstraint {
                ..Default::default()
            },
        );
    }

    let previous_pass_color = if graph_config.enable_hdr {
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

    graph_callbacks.set_begin_execute_graph_callback(move |args| {
        let mut write_context =
            RenderJobBeginExecuteGraphContext::from_on_begin_execute_graph_args(&args);
        args.graph_context.prepared_render_data().on_begin_execute_graph(&mut write_context)
    });

    //
    // Create the executor, it needs to have access to the resource manager to add framebuffers
    // and renderpasses to the resource lookups
    //
    let executor = RenderGraphExecutor::new(
        &device_context,
        &resource_context,
        graph,
        &swapchain_resources.swapchain_surface_info,
        graph_callbacks,
    )?;

    render_resources.fetch_mut::<ShadowMapResource>().set_shadow_map_image_views(&executor);

    Ok(BuildRenderGraphResult {
        executor,
    })
}
