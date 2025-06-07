use std::{path::PathBuf, sync::Arc};

use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum SummaryDestination {
    /// Summary will be placed inline in the file
    Inline(PathBuf),
    /// Summary will be placed in a new file at the specified path
    NewFile(PathBuf),
}

#[derive(Debug, Clone)]
pub struct ADPrompt(pub String);

impl ADPrompt {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ADSummary(String);

impl ADSummary {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum PromptSection {
    StaticString(String),
    Placeholder(Arc<Mutex<ExecutionStep>>)
}

#[derive(Debug, Clone)]
pub struct ExecutionStepContext {
    /// The prompt sections used to build the prompt
    pub prompt_sections: Vec<PromptSection>,
    /// The built prompt
    pub prompt: Option<Arc<ADPrompt>>,
    /// The generated summary
    pub summary: Option<Arc<ADSummary>>,
    /// The target path for this step
    pub target_path: PathBuf,
    /// Where to place the summary
    pub summary_destination: SummaryDestination,
    /// Hash of the previous prompt (for caching)
    pub previous_prompt_hash: String,
}

#[derive(Debug, Clone)]
pub enum ExecutionStep {
    AwaitingPrompt(ExecutionStepContext),
    PendingExecution(ExecutionStepContext),
    Executing(ExecutionStepContext),
    PendingSave(ExecutionStepContext),
    Done(ExecutionStepContext),
    Placeholder
}

impl ExecutionStep {

    /// Turn `AwaitingPrompt` → `PendingExecution`, consuming `self`
    /// and mutating the Context on the way.
    pub fn into_pending_exec(mut self, prompt: ADPrompt) -> Self {
        match self {
            ExecutionStep::AwaitingPrompt(mut ctx) => {
                // --- mutate ctx here -----------------------------------------
                ctx.prompt = Some(Arc::new(prompt));
                //----------------------------------------------------------------
                ExecutionStep::PendingExecution(ctx)
            }
            other => panic!(
                "Invalid state: expected AwaitingPrompt, got {:?}",
                other
            ),  
        }
    }

    /// Turn `PendingExecution` → `PendingSave` in one go.
    pub fn into_executing(mut self) -> Self {
        match self {
            ExecutionStep::PendingExecution(ctx) => {
                ExecutionStep::Executing(ctx)
            }
            other => panic!(
                "Invalid state: expected PendingExecution, got {:?}",
                other
            ),
        }
    }

    /// Turn `PendingExecution` → `PendingSave` in one go.
    pub fn into_pending_save(mut self, summary: ADSummary) -> Self {
        match self {
            ExecutionStep::PendingExecution(mut ctx) => {
                ctx.summary = Some(Arc::new(summary));
                ExecutionStep::PendingSave(ctx)
            }
            other => panic!(
                "Invalid state: expected Executing, got {:?}",
                other
            ),
        }
    }

    /// In-place state advance (no heap alloc) using `&mut self`
    pub fn into_done(mut self) -> Self {
        match self {
            ExecutionStep::PendingSave(mut ctx) => {
                ExecutionStep::Done(ctx)
            }
            other => panic!(
                "Invalid state: expected PendingSave, got {:?}",
                other
            ),
        }
    }
}

/* ---------- main Default for ExecutionStep --------------------------- */

impl Default for ExecutionStep {
    fn default() -> Self {
        ExecutionStep::Placeholder
    }
}