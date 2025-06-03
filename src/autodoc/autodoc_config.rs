#[derive(Debug)]
pub struct AutodocConfig {
    pub mode: SummaryMode,
    pub ignore_paths: Vec<String>
}

#[derive(Debug, Copy, Clone)]
pub enum SummaryMode {
    Inline,
    Shadow,
    Adjacent,
}