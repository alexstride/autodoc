// TODO: remove!
#![allow(warnings)]


pub mod crawler;
pub mod tree_renderer;
pub mod common;
pub mod autodoc;

use clap::{Parser, ValueHint};
use std::{collections::HashMap, path::PathBuf, sync::{Arc}, thread, time::Duration};
use tokio::sync::Mutex;
// use crawler::{Crawler, Directory};
// use tree_renderer::render_directory_tree;
use autodoc::{AutodocConfig};

const CONFIG_FILE_NAME: &str = "autodoc.config";

/// List child files and/or directories of the given path.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Target path (file or directory)
    #[arg(value_hint = ValueHint::AnyPath)]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let path = cli.path.clone();
    // run_autodoc_session(start_path);
    println!("{}", path.join("fooooo").display());
    Ok(())
}

fn run_autodoc_session(start_path: PathBuf) -> () {
    let root_path: PathBuf = locate_parent_dir_with(CONFIG_FILE_NAME, &start_path);
    let config_path = root_path.join(CONFIG_FILE_NAME);
    let config: AutodocConfig = load_config(config_path);
    let repo = load_repo(root_path, &config);
    let plan: ADPlan = create_plan(repo, &start_path, &config);

    // Todo: put in a proper function for the get_summary function
    // execute_plan(plan, &config, on_repo_update, |_| {ADSummary(String::from("hello"))});
}

fn locate_parent_dir_with(filename: &str, start_path: &PathBuf) -> PathBuf {
    todo!("Todo: implement function to recurse upwards in the file tree and find the closest ancestor \
containing a file with the given filename");
}

fn load_config(config_path: PathBuf) -> AutodocConfig {
    todo!("Implement function to load config from the autodoc.config file (yaml format)");
}

// This is going to be immutable
struct ADRepo {
    root_path: PathBuf,
    tree: TreeNode
}

enum TreeNode {
    Dir(ADDir),
    File(ADFile),
}

struct ADDir {
    path: PathBuf,
    children: Vec<TreeNode>,
    summary: Option<ADSummary>
}

struct ADFile {
    path: PathBuf,
    size: u64,
    summary: Option<ADSummary>
}

#[derive(Debug, Clone)]
struct ADPrompt(Arc<String>);

impl ADPrompt {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Debug, Clone)]
struct ADSummary(Arc<String>);

impl ADSummary {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

type ADPlan = Vec<ExecutionStep>;

fn load_repo(repo_root: PathBuf, config: &AutodocConfig) -> ADRepo {
    todo!("implement a function which crawls the whole repo and loads it into the ADRepo dataclass")
}

struct TreeNodePlan {
    
}

fn create_plan(repo: ADRepo, start_path: &PathBuf, config: &AutodocConfig) -> ADPlan {
    todo!("Implement a function which takes the repo and the start path and uses it to create an execution plan")
}

enum PromptSection {
    StaticString(String),
    Placeholder(Arc<Mutex<ExecutionStep>>)
}

enum ExecutionStep {
    AwaitingPrompt(Vec<PromptSection>),
    PendingExecution(ADPrompt),
    Executing (ADPrompt),
    PendingSave(ADSummary),
    Done(ADSummary)
}

async fn execute_plan<SummaryFut>(
    mut plan: Vec<ExecutionStep>,
    config: &AutodocConfig,
    on_update: fn(ADRepo) -> (),
    get_summary: fn(&str) -> SummaryFut
) where SummaryFut: std::future::Future<Output = ADSummary> + Send + Sync + 'static {
    let mut is_complete = false;
    // Assume that the ADPlan is a list of execution steps. We have taken ownership of the plan and will use it
    // as the internal list

    let mut mutable_steps: Vec<Arc<Mutex<ExecutionStep>>> = plan
        .into_iter()                      // move every ExecutionStep out of `plan`
        .map(|step| Arc::new(Mutex::new(step)))
        .collect();

    let mut done_steps: usize = 0;

    while (!is_complete) {
        thread::sleep(Duration::from_millis(100));
        for step in &mut mutable_steps {
            let mut step_guard = step.lock().await;
            match &mut *step_guard {
                ExecutionStep::AwaitingPrompt(prompt_sections) => {
                    let mut remaining_placeholders = 0;
                    // Try to resolve any placeholders in the prompt
                    // If the prompt section is a string, nothing to do
                    // If the prompt section is a placeholder, holding a pointer, follow the pointer
                    // to check if the execution step is PendingSave or Done, and if so, resolve the prompt string
                    for section in prompt_sections.iter_mut() {
                        match section {
                            PromptSection::StaticString(_) => (),
                            PromptSection::Placeholder(wrapped_step_pointer) => {
                                let resolved_text = {
                                    let inner_step_guard = wrapped_step_pointer.lock().await;
                                    match &*inner_step_guard {
                                        ExecutionStep::PendingSave(summary) | ExecutionStep::Done(summary) => Some(summary.as_str().to_string()),
                                        _ => None
                                    }
                                };
                                if let Some(text) = resolved_text {
                                    *section = PromptSection::StaticString(text)
                                } else {
                                    remaining_placeholders += 1;
                                }
                            },
                        }
                    }
                    if remaining_placeholders == 0 {
                        let prompt = join_prompt_sections(&prompt_sections);
                        *step_guard = ExecutionStep::PendingExecution(prompt)
                    }
                },
                ExecutionStep::PendingExecution(prompt) => {
                    // Clone what the async task needs *before* we move out
                    let prompt_clone = prompt.clone();
                    let step_clone   = step.clone();
                    let on_update    = &on_update;
                    let get_summary  = &get_summary;

                    // Spawn a detached task – it will update the step later
                    tokio::spawn(async move {
                        let summary = get_summary(prompt_clone.as_str()).await;

                        // 2a) Mark the step as PendingSave
                        {
                            let mut s = step_clone.lock().await;
                            *s = ExecutionStep::PendingSave(summary);
                        }

                        // TODO: Call on_update with the update
                    });
                    *step_guard = ExecutionStep::Executing(prompt.clone());
                },
                ExecutionStep::Executing(_) => (),
                ExecutionStep::PendingSave(summary) => {
                    // 1. Persist the summary (await inside the loop is fine: we hold
                    //    the mutex only around the assignment below, not the write).
                    if let Err(e) = persist_summary(summary, config).await {
                        eprintln!("failed to save summary: {e}");
                        continue;          // try again next tick
                    }
    
                    // 2. Notify the outside world.
                    // TODO: Make call the update callback
                    // on_update(repo_after_save(summary));
    
                    // 3. Move the summary into Done
                    *step_guard = ExecutionStep::Done(summary.clone());
                },
                ExecutionStep::Done(_) => {
                    done_steps += 1;
                }
            }
        }
        if done_steps == mutable_steps.len() {
            is_complete = true;
        }
    }
    // TODO: Make sure that the control loop backs off if nothing has been progressed this tick
}



fn join_prompt_sections(sections: &[PromptSection]) -> ADPrompt {
    let combined = sections
        .iter()
        .map(|section| {
            if let PromptSection::StaticString(s) = section {
                s
            } else {
                panic!("Unresolved placeholder in prompt sections")
            }
        })
        .cloned()
        .collect::<Vec<_>>()
        .join("\n\n");
    ADPrompt(combined.into())
}

fn on_repo_update(repo: ADRepo) -> () {
    todo!("Implement a function to render the new repo")
}

async fn persist_summary(
    summary: &ADSummary,
    config: &AutodocConfig,
) -> anyhow::Result<()> {
    // Derive an output path from your config (adapt as you like)
    // let dir: &PathBuf = &config.out_dir;          // <- add this field to `AutodocConfig`
    // fs::create_dir_all(dir).await?;

    // let file_name = format!("{}.md", uuid::Uuid::new_v4());
    // let path = dir.join(file_name);

    // fs::write(path, summary.as_str()).await?;
    // Ok(())
    Ok(())
}