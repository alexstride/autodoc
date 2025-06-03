use std::path::{PathBuf};
use crate::autodoc::autodoc_config::{AutodocConfig};
use crate::autodoc::default_config_provider::load_config;
use crate::file_crawler::{FileCrawler, TreeNode};

use anyhow::Result;

const CONFIG_FILE_NAME: &str = "autodoc.config";

type RenderFn = fn(&TreeNode) -> Result<()>;

// ─────────────────────────────────────────────────────────────────────────────
// AutodocSession
// ─────────────────────────────────────────────────────────────────────────────

pub struct AutodocSession<C>
where
    C: FileCrawler
{
    crawler: C,
    renderer: RenderFn,
    repo_root: PathBuf,
    config: AutodocConfig,
    target_tree: TreeNode,
}

impl<C> AutodocSession<C>
where
    C: FileCrawler
    {
    /// Construct a new session. Prefer explicit dependency injection so the
    /// whole unit test can be kept in-process with mocks.
    pub fn new(
        target: PathBuf,
        crawler: C,
        renderer: RenderFn,
        config: Option<AutodocConfig>
    ) -> Self {
        let config = config.unwrap_or_else(|| 
            load_config(&target, &crawler).expect("Failed to load config!")
        );
        let target_tree = crawler.crawl(&target, None, true).unwrap();
        Self {
            crawler,
            renderer,
            repo_root: target,
            config,
            target_tree,
        }
    }

    /// Entry-point called from `main.rs`. Orchestrates the four high-level
    /// steps described in the design doc.
    pub fn run(&mut self) -> Result<()> {
        // TODO: 
        // 1) Construct a new directory structure which encodes information about any autodoc summaries which can be discovered
        // This will look a bit like the original directory tree which gets passed in, but it will have information on the nodes
        // like whether or not a summary document is present, a hash of all of the content being hashed, and a hash of the prompt
        // created (for detecting changes)
        // 2) Use the tree to create a plan.
            // Create autodoc_plan file and define a struct called ADPlan which contains a vec of ADPlanStage and a total number of
            // estimated tokens to execute the plan. Expose a create_plan method which takes in an ADTreeNode and returns an ADPlan
            // ADPlanStage will have to be an enum of FileSummarise and DirSummarise, which we can leave fairly empty for now.
            // Both of them will have to implement a trait which will allow the plan to be executed, as well as a trait which gives
            // a token estimate

        // 3) Execute plan - This can just be a noop for now, but it should be a function which comes from a plan_executor function
        

        // Temporarily just print the file tree, untill we have something more advanced to achieve
        // Eventually we will want to be animating the progress of the whole process
        (self.renderer)(&self.target_tree).expect("Rendering failed!");
        Ok(())
    }
}
