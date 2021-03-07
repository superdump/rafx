use crate::phases::TransparentRenderPhase;
use crate::phases::{OpaqueRenderPhase, UiRenderPhase};
use crate::time::TimeState;
use rafx::assets::distill_impl::AssetResource;
use rafx::assets::{image_upload, AssetManagerRenderResource, GpuImageDataColorSpace};
use rafx::assets::{AssetManager, GpuImageData};
use rafx::framework::{DynResourceAllocatorSet, RenderResources};
use rafx::framework::{ImageViewResource, ResourceArc};
use rafx::nodes::{
    ExtractJobSet, ExtractResources, FramePacketBuilder, RenderJobExtractContext,
    RenderNodeReservations, RenderPhaseMaskBuilder, RenderView,
    RenderViewDepthRange, RenderViewSet,
};
use rafx::visibility::{DynamicVisibilityNodeSet, StaticVisibilityNodeSet};
use std::sync::{Arc, Mutex};

use super::*;

use rafx::api::extra::upload::{RafxTransferUpload, RafxUploadError};
use rafx::api::{
    RafxDeviceContext, RafxError, RafxPresentableFrame, RafxQueue, RafxResourceType, RafxResult,
    RafxSwapchainHelper,
};
use rafx::assets::image_upload::ImageUploadParams;
use crate::features::mesh::shadow_map_resource::ShadowMapResource;


#[derive(Clone)]
pub struct InvalidResources {
    pub invalid_image: ResourceArc<ImageViewResource>,
    pub invalid_cube_map_image: ResourceArc<ImageViewResource>,
}

pub struct GameRendererInner {
    pub(super) invalid_resources: InvalidResources,

    // Everything that is loaded all the time
    pub(super) static_resources: GameRendererStaticResources,

    // Everything that requires being created after the swapchain inits
    pub(super) swapchain_resources: Option<SwapchainResources>,

    pub(super) render_thread: RenderThread,
    pub(super) plugins: Vec<Box<dyn RendererPlugin>>,
}

#[derive(Clone)]
pub struct GameRenderer {
    pub(super) inner: Arc<Mutex<GameRendererInner>>,
    pub(super) graphics_queue: RafxQueue,
    pub(super) transfer_queue: RafxQueue,
}

impl GameRenderer {
    pub fn new(
        extract_resources: ExtractResources,
        asset_resource: &mut AssetResource,
        asset_manager: &mut AssetManager,
        graphics_queue: &RafxQueue,
        transfer_queue: &RafxQueue,
        mut plugins: Vec<Box<dyn RendererPlugin>>,
    ) -> RafxResult<Self> {
        let device_context = graphics_queue.device_context();

        let dyn_resource_allocator = asset_manager.create_dyn_resource_allocator_set();

        let mut upload = RafxTransferUpload::new(
            &device_context,
            asset_manager.transfer_queue(),
            asset_manager.graphics_queue(),
            16 * 1024 * 1024,
        )?;

        let invalid_image = Self::upload_image_data(
            &device_context,
            &mut upload,
            &dyn_resource_allocator,
            &GpuImageData::new_1x1_rgba8(255, 0, 255, 255, GpuImageDataColorSpace::Linear),
            ImageUploadParams::default(),
        )
        .map_err(|x| Into::<RafxError>::into(x))?;

        let invalid_cube_map_image = Self::upload_image_data(
            &device_context,
            &mut upload,
            &dyn_resource_allocator,
            &GpuImageData::new_1x1_rgba8(255, 0, 255, 255, GpuImageDataColorSpace::Linear),
            ImageUploadParams {
                generate_mips: false,
                resource_type: RafxResourceType::TEXTURE_CUBE,
                layer_swizzle: Some(&[0, 0, 0, 0, 0, 0]),
            },
        )
        .map_err(|x| Into::<RafxError>::into(x))?;

        let static_resources = GameRendererStaticResources::new(asset_resource, asset_manager)?;

        let mut render_resources = RenderResources::default();
        for plugin in &mut plugins {
            plugin.initialize_static_resources(
                asset_manager,
                asset_resource,
                &extract_resources,
                &mut render_resources,
                &mut upload,
            )?;
        }

        upload.block_until_upload_complete()?;

        let render_thread = RenderThread::start(render_resources);

        let renderer = GameRendererInner {
            invalid_resources: InvalidResources {
                invalid_image,
                invalid_cube_map_image,
            },
            static_resources,
            swapchain_resources: None,

            plugins,
            render_thread,
        };

        Ok(GameRenderer {
            inner: Arc::new(Mutex::new(renderer)),
            graphics_queue: graphics_queue.clone(),
            transfer_queue: transfer_queue.clone(),
        })
    }

    pub fn graphics_queue(&self) -> &RafxQueue {
        &self.graphics_queue
    }

    pub fn transfer_queue(&self) -> &RafxQueue {
        &self.transfer_queue
    }

    fn upload_image_data(
        device_context: &RafxDeviceContext,
        upload: &mut RafxTransferUpload,
        dyn_resource_allocator: &DynResourceAllocatorSet,
        image_data: &GpuImageData,
        params: ImageUploadParams,
    ) -> Result<ResourceArc<ImageViewResource>, RafxUploadError> {
        let texture = image_upload::enqueue_load_image(device_context, upload, image_data, params)?;

        let image = dyn_resource_allocator.insert_texture(texture);

        Ok(dyn_resource_allocator.insert_image_view(&image, None)?)
    }

    // This is externally exposed, it checks result of the previous frame (which implicitly also
    // waits for the previous frame to complete if it hasn't already)
    #[profiling::function]
    pub fn start_rendering_next_frame(
        &self,
        extract_resources: &mut ExtractResources,
        window_width: u32,
        window_height: u32,
    ) -> RafxResult<()> {
        //
        // Block until the previous frame completes being submitted to GPU
        //
        let t0 = std::time::Instant::now();

        let presentable_frame = {
            let mut swapchain_helper = extract_resources.fetch_mut::<RafxSwapchainHelper>();
            let mut asset_manager = extract_resources.fetch_mut::<AssetManager>();
            SwapchainHandler::acquire_next_image(
                &mut *swapchain_helper,
                &mut *asset_manager,
                self,
                window_width,
                window_height,
            )
        }?;

        self.inner
            .lock()
            .unwrap()
            .render_thread
            .wait_for_render_finish(std::time::Duration::from_secs(30));

        let t1 = std::time::Instant::now();
        log::trace!(
            "[main] wait for previous frame present {} ms",
            (t1 - t0).as_secs_f32() * 1000.0
        );

        Self::create_and_start_render_job(
            self,
            extract_resources,
            window_width,
            window_height,
            presentable_frame,
        );

        Ok(())
    }

    fn create_and_start_render_job(
        game_renderer: &GameRenderer,
        extract_resources: &mut ExtractResources,
        window_width: u32,
        window_height: u32,
        presentable_frame: RafxPresentableFrame,
    ) {
        let result = Self::try_create_render_job(
            &game_renderer,
            extract_resources,
            window_width,
            window_height,
            &presentable_frame,
        );

        let mut guard = game_renderer.inner.lock().unwrap();
        let game_renderer_inner = &mut *guard;
        match result {
            Ok(prepared_frame) => game_renderer_inner
                .render_thread
                .render(prepared_frame, presentable_frame),
            Err(e) => {
                let graphics_queue = game_renderer.graphics_queue();
                presentable_frame.present_with_error(graphics_queue, e)
            }
        };
    }

    fn try_create_render_job(
        game_renderer: &GameRenderer,
        extract_resources: &mut ExtractResources,
        window_width: u32,
        window_height: u32,
        presentable_frame: &RafxPresentableFrame,
    ) -> RafxResult<RenderFrameJob> {
        //
        // Fetch resources
        //
        let mut static_visibility_node_set_fetch =
            extract_resources.fetch_mut::<StaticVisibilityNodeSet>();
        let static_visibility_node_set = &mut *static_visibility_node_set_fetch;

        let mut dynamic_visibility_node_set_fetch =
            extract_resources.fetch_mut::<DynamicVisibilityNodeSet>();
        let dynamic_visibility_node_set = &mut *dynamic_visibility_node_set_fetch;

        let mut asset_manager_fetch = extract_resources.fetch_mut::<AssetManager>();
        let asset_manager = &mut *asset_manager_fetch;

        let render_registry = asset_manager.resource_manager().render_registry().clone();
        let device_context = asset_manager.device_context().clone();

        //
        // Mark the previous frame as completed
        //
        asset_manager.on_frame_complete()?;

        let resource_context = asset_manager.resource_manager().resource_context();

        let mut guard = game_renderer.inner.lock().unwrap();
        let game_renderer_inner = &mut *guard;
        let render_resources = &mut game_renderer_inner
            .render_thread
            .render_resources()
            .lock()
            .unwrap();

        let static_resources = &game_renderer_inner.static_resources;
        render_resources.insert(static_resources.clone());
        render_resources.insert(game_renderer_inner.invalid_resources.clone());

        //
        // Swapchain Status
        //
        let swapchain_resources = game_renderer_inner.swapchain_resources.as_mut().unwrap();

        let swapchain_image = {
            // Temporary hack to jam a swapchain image into the existing resource lookups.. may want
            // to reconsider this later since the ResourceArc can be held past the lifetime of the
            // swapchain image
            let swapchain_image = presentable_frame.swapchain_texture().clone();

            let swapchain_image = resource_context.resources().insert_image(swapchain_image);

            resource_context
                .resources()
                .get_or_create_image_view(&swapchain_image, None)?
        };

        let swapchain_surface_info = swapchain_resources.swapchain_surface_info.clone();


        //
        // Build the frame packet - this takes the views and visibility results and creates a
        // structure that's used during the extract/prepare/write phases
        //
        let frame_packet_builder = {
            let mut render_node_reservations = RenderNodeReservations::default();
            for plugin in &game_renderer_inner.plugins {
                plugin.add_render_node_reservations(&mut render_node_reservations, extract_resources);
            }

            FramePacketBuilder::new(&render_node_reservations)
        };

        let render_view_set = RenderViewSet::default();

        //
        // Determine Camera Location
        //
        let main_view = GameRenderer::calculate_main_view(
            &render_view_set,
            window_width,
            window_height,
            extract_resources,
        );

        //
        // Visibility
        //
        let main_view_static_visibility_result =
            static_visibility_node_set.calculate_static_visibility(&main_view);
        let main_view_dynamic_visibility_result =
            dynamic_visibility_node_set.calculate_dynamic_visibility(&main_view);

        log::trace!(
            "main view static node count: {}",
            main_view_static_visibility_result.handles.len()
        );

        log::trace!(
            "main view dynamic node count: {}",
            main_view_dynamic_visibility_result.handles.len()
        );



        // After these jobs end, user calls functions to start jobs that extract data
        frame_packet_builder.add_view(
            &main_view,
            &[
                main_view_static_visibility_result,
                main_view_dynamic_visibility_result,
            ],
        );

        {
            let mut shadow_map_resource = render_resources.fetch_mut::<ShadowMapResource>();
            shadow_map_resource.recalculate_shadow_map_views(&render_view_set, extract_resources, &frame_packet_builder, static_visibility_node_set, dynamic_visibility_node_set);
        }

        let frame_packet = frame_packet_builder.build();

        //
        // Update Resources and flush descriptor set changes
        //
        asset_manager.on_begin_frame()?;

        //
        // Extract Jobs
        //
        let mut extract_jobs = Vec::default();
        for plugin in &game_renderer_inner.plugins {
            plugin.add_extract_jobs(&extract_resources, render_resources, &mut extract_jobs);
        }

        let extract_job_set = ExtractJobSet::new(extract_jobs);

        //
        //
        //
        render_resources.insert(swapchain_surface_info.clone());
        unsafe {
            render_resources.insert(AssetManagerRenderResource::new(asset_manager));
        }

        let prepare_job_set = {
            profiling::scope!("renderer extract");

            let extract_context =
                RenderJobExtractContext::new(&extract_resources, &render_resources);

            let mut extract_views = Vec::default();
            extract_views.push(main_view.clone());

            let shadow_map_resource = render_resources.fetch::<ShadowMapResource>();
            shadow_map_resource.append_render_views(&mut extract_views);

            extract_job_set.extract(&extract_context, &frame_packet, &extract_views)
        };

        render_resources.remove::<AssetManagerRenderResource>();

        //TODO: This is now possible to run on the render thread
        let render_graph = render_graph::build_render_graph(
            &device_context,
            &resource_context,
            asset_manager,
            swapchain_image,
            main_view.clone(),
            swapchain_resources,
            static_resources,
            extract_resources,
            render_resources,
        )?;

        let game_renderer = game_renderer.clone();
        let graphics_queue = game_renderer.graphics_queue.clone();

        let prepared_frame = RenderFrameJob {
            game_renderer,
            prepare_job_set,
            render_graph: render_graph.executor,
            resource_context,
            frame_packet,
            main_view,
            render_registry,
            device_context,
            graphics_queue,
        };

        Ok(prepared_frame)
    }

    #[profiling::function]
    fn calculate_main_view(
        render_view_set: &RenderViewSet,
        window_width: u32,
        window_height: u32,
        extract_resources: &ExtractResources,
    ) -> RenderView {
        let time_state_fetch = extract_resources.fetch::<TimeState>();
        let time_state = &*time_state_fetch;

        let main_camera_render_phase_mask = RenderPhaseMaskBuilder::default()
            .add_render_phase::<OpaqueRenderPhase>()
            .add_render_phase::<TransparentRenderPhase>()
            .add_render_phase::<UiRenderPhase>()
            .build();

        const CAMERA_XY_DISTANCE: f32 = 12.0;
        const CAMERA_Z: f32 = 6.0;
        const CAMERA_ROTATE_SPEED: f32 = -0.10;
        const CAMERA_LOOP_OFFSET: f32 = -0.3;
        let loop_time = time_state.total_time().as_secs_f32();
        let eye = glam::Vec3::new(
            CAMERA_XY_DISTANCE * f32::cos(CAMERA_ROTATE_SPEED * loop_time + CAMERA_LOOP_OFFSET),
            CAMERA_XY_DISTANCE * f32::sin(CAMERA_ROTATE_SPEED * loop_time + CAMERA_LOOP_OFFSET),
            CAMERA_Z,
        );

        let aspect_ratio = window_width as f32 / window_height as f32;

        let view = glam::Mat4::look_at_rh(eye, glam::Vec3::zero(), glam::Vec3::new(0.0, 0.0, 1.0));

        let near_plane = 0.01;
        let proj = glam::Mat4::perspective_infinite_reverse_rh(
            std::f32::consts::FRAC_PI_4,
            aspect_ratio,
            near_plane,
        );

        render_view_set.create_view(
            eye,
            view,
            proj,
            (window_width, window_height),
            RenderViewDepthRange::new_infinite_reverse(near_plane),
            main_camera_render_phase_mask,
            "main".to_string(),
        )
    }
}
