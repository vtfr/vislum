
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineLayoutDescription {
    push_constants_range: Vec<Range<u32>>,
}

pub struct PipelineLayoutCache {

}
