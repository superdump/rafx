use crate::{FramePacket, PrepareJob, PrepareJobSet, RenderFeatureIndex, RenderView};

pub trait ExtractJob<ExtractContextT, PrepareContextT, WriteContextT> {
    fn extract(
        self: Box<Self>,
        extract_context: &ExtractContextT,
        frame_packet: &FramePacket,
        views: &[&RenderView],
    ) -> Box<dyn PrepareJob<PrepareContextT, WriteContextT>>;

    fn feature_debug_name(&self) -> &'static str;
    fn feature_index(&self) -> RenderFeatureIndex;
}

pub struct ExtractJobSet<ExtractContextT, PrepareContextT, WriteContextT> {
    extract_jobs: Vec<Box<dyn ExtractJob<ExtractContextT, PrepareContextT, WriteContextT>>>,
}

impl<ExtractContextT, PrepareContextT, WriteContextT> Default
    for ExtractJobSet<ExtractContextT, PrepareContextT, WriteContextT>
{
    fn default() -> Self {
        ExtractJobSet {
            extract_jobs: Default::default(),
        }
    }
}

impl<ExtractContextT, PrepareContextT, WriteContextT>
    ExtractJobSet<ExtractContextT, PrepareContextT, WriteContextT>
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_job(
        &mut self,
        extract_job: Box<dyn ExtractJob<ExtractContextT, PrepareContextT, WriteContextT>>,
    ) {
        self.extract_jobs.push(extract_job)
    }

    pub fn extract(
        self,
        extract_context: &ExtractContextT,
        frame_packet: &FramePacket,
        views: &[&RenderView],
    ) -> PrepareJobSet<PrepareContextT, WriteContextT> {
        log::trace!("Start extract job set");

        let mut prepare_jobs = vec![];
        for extract_job in self.extract_jobs {
            log::trace!("Start job {}", extract_job.feature_debug_name());

            let prepare_job = extract_job.extract(extract_context, frame_packet, views);
            prepare_jobs.push(prepare_job);
        }

        PrepareJobSet::new(prepare_jobs)
    }
}
