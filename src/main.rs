pub mod crawler;
pub mod tree_renderer;
pub mod common;
pub mod autodoc;

use clap::{Parser, ValueHint};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};
// use crawler::{Crawler, Directory};
// use tree_renderer::render_directory_tree;
use autodoc::{AutodocConfig, ExecutionStep, ADPrompt, ADSummary, PromptSection};

const CONFIG_FILE_NAME: &str = "autodoc.config";

type ADPlan = Vec<ExecutionStep>;

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

fn load_repo(repo_root: PathBuf, config: &AutodocConfig) -> ADRepo {
    todo!("implement a function which crawls the whole repo and loads it into the ADRepo dataclass")
}

struct TreeNodePlan {
    
}

fn create_plan(repo: ADRepo, start_path: &PathBuf, config: &AutodocConfig) -> ADPlan {
    todo!("Implement a function which takes the repo and the start path and uses it to create an execution plan")
}


async fn execute_plan<SummaryFut, SaveFut>(
    plan: ADPlan,
    config: &AutodocConfig,
    on_update: fn(ADRepo) -> (),
    get_summary: fn(&str) -> SummaryFut,
    save_summary: fn(&str) -> SaveFut,
) where 
    SummaryFut: std::future::Future<Output = ADSummary> + Send + Sync + 'static,
    SaveFut: Future<Output = anyhow::Result<()>> + Send + 'static
{
    let mut mutable_steps: Vec<Arc<Mutex<ExecutionStep>>> = plan
        .into_iter()                      // move every ExecutionStep out of `plan`
        .map(|step| Arc::new(Mutex::new(step)))
        .collect();

    loop {
        let TickResult {
            task_complete,
            steps_progressed,
            step_errors,
        } = tick(&mut mutable_steps, on_update, get_summary, save_summary).await;

        if task_complete {
            break;
        }

        // nothing advanced and no new errors? back-off a bit
        if steps_progressed == 0 && step_errors == 0 {
            sleep(Duration::from_millis(200)).await;
        }
    }
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


struct TickResult {
    task_complete: bool,
    steps_progressed: usize,
    step_errors: usize
}
async fn tick<SummaryFut, SaveFut>(
    steps: &mut Vec<Arc<Mutex<ExecutionStep>>>,
    on_update: fn(ADRepo) -> (),
    get_summary: fn(&str) -> SummaryFut,
    save_summary: fn(&str) -> SaveFut,
) -> TickResult  
where 
    SummaryFut: std::future::Future<Output = ADSummary> + Send + Sync + 'static,
    SaveFut: Future<Output = anyhow::Result<()>> + Send + 'static 
{
    let mut done_steps: usize = 0;
    let mut steps_progressed: usize = 0;
    for step in &mut *steps {
        let mut step_guard = step.lock().await;
        match &mut *step_guard {
            ExecutionStep::AwaitingPrompt(context) => {
                let mut remaining_placeholders = 0;
                // Resolve placeholders in the prompt sections
                for section in context.prompt_sections.iter_mut() {
                    match section {
                        PromptSection::StaticString(_) => (),
                        PromptSection::Placeholder(wrapped_step_pointer) => {
                            let resolved_text = {
                                let inner_step_guard = wrapped_step_pointer.lock().await;
                                match &*inner_step_guard {
                                    ExecutionStep::PendingSave(summary_ctx) | ExecutionStep::Done(summary_ctx) => {
                                        summary_ctx.summary.as_ref().map(|s| s.as_str().to_string())
                                    },
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
                    let prompt = join_prompt_sections(&context.prompt_sections);
                    let awaiting = std::mem::take(&mut *step_guard);
                    *step_guard = awaiting.into_pending_exec(prompt);
                    steps_progressed += 1;
                }
            },
            ExecutionStep::PendingExecution(context) => {
                let prompt = context.prompt.as_ref().expect("Prompt should be set in PendingExecution state");
                let prompt_clone = prompt.clone();
                let step_clone = step.clone();

                // Spawn a detached task to get the summary
                tokio::spawn(async move {
                    let summary = get_summary(prompt_clone.as_str()).await;
                    
                    let mut s = step_clone.lock().await;
                    let awaiting = std::mem::take(&mut *s);
                    *s = awaiting.into_pending_save(summary);
                });
                let awaiting = std::mem::take(&mut *step_guard);
                *step_guard = awaiting.into_executing();
                steps_progressed += 1;
            },
            ExecutionStep::Executing(_) => (),
            ExecutionStep::PendingSave(context) => {
                let summary = context.summary.as_ref().expect("Summary should be set in PendingSave state");
                
                // Persist the summary
                if let Err(e) = save_summary(summary.as_str()).await {
                    eprintln!("failed to save summary: {e}");
                    continue;  // try again next tick
                }
                
                // Notify the outside world
                // TODO: Call on_update with the update
                // on_update(repo_after_save(summary));

                let awaiting = std::mem::take(&mut *step_guard);
                *step_guard = awaiting.into_done();
                steps_progressed += 1;
                done_steps += 1;
            },
            ExecutionStep::Done(_) => {
                done_steps += 1;
            },
            ExecutionStep::Placeholder => { panic!("Unexpected placeholder encountered. Illegal app state")}
        }
    }
    let is_complete = done_steps == steps.len();
    TickResult { task_complete: is_complete, steps_progressed, step_errors: 0}
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