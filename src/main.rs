// TODO: remove!
#![allow(warnings)]


pub mod crawler;
pub mod tree_renderer;
pub mod common;
pub mod autodoc;

use clap::{Parser, ValueHint};
use std::{collections::HashMap, path::PathBuf, sync::{Arc, Mutex}, thread, time::Duration};
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
    execute_plan(plan, &config, on_repo_update, |_| {ADSummary(String::from("hello"))});
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

#[derive(Debug, Clone)]
struct ADPrompt(String);
#[derive(Debug, Clone)]
struct ADSummary(String);


#[derive(Debug)]
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

// fn execute_plan(
//     mut plan: Vec<ExecutionStep>,
//     config: &AutodocConfig,
//     on_update: fn(ADRepo) -> (),
//     get_summary: fn(String) -> ADSummary
// ) -> () {
//     let is_complete = false;
//     // Assume that the ADPlan is a list of execution steps. We have taken ownership of the plan and will use it
//     // as the internal 
//     // Loop over all of the values in the hashmap and find the first one which is in a state which can be progressed
//     while (!is_complete) {
//         thread::sleep(Duration::from_millis(100));
//         for step in &mut plan {
//             match step {
//                 ExecutionStep::AwaitingPrompt(prompt_sections) => {
//                     let mut remaining_placeholders = 0;
//                     // Try to resolve any placeholders in the prompt
//                     // If the prompt section is a string, nothing to do
//                     // If the prompt section is a placeholder, holding a pointer, follow the pointer
//                     // to check if the execution step is PendingSave or Done, and if so, resolve the prompt string
//                     for section in prompt_sections {
//                         match section {
//                             PromptSection::StaticString(_) => (),
//                             PromptSection::Placeholder(arc_mutex_step) => {
//                                 let step_guard = arc_mutex_step.lock().unwrap();
//                                 match &*step_guard {
//                                     ExecutionStep::PendingSave(summary) | ExecutionStep::Done(summary) => {
//                                         *section = PromptSection::StaticString(summary.0.clone())
//                                     }
//                                     _ => remaining_placeholders += 1
//                                 }
//                             },
//                         }
//                     }
//                     if remaining_placeholders == 0 {
//                         let prompt = join_prompt_sections(&prompt_sections);
//                         *step = ExecutionStep::PendingExecution(prompt)
//                     }
//                 },
//                 ExecutionStep::PendingExecution(prompt) => {
//                     // TODO
//                     // Kick off a request to get a summary async, but then move on. Queue up a callback to update the step when 
//                 }
//             }
//         }
//     }
// }


/// Signature:
///   * `plan`        – shared, mutable execution steps
///   * `config`      – unchanged, read-only
///   * `on_update`   – called each time a step reaches `Done`
///   * `get_summary` – async closure that talks to your LLM
pub async fn execute_plan<
    FGet, FutGet, 
    FUpdate
>(
    plan: ADPlan,
    _config: Arc<AutodocConfig>,
    on_update: FUpdate,
    get_summary: FGet,
) 
where
    // async get-summary callback
    FGet:     Fn(String) -> FutGet + Send + Sync + 'static,
    FutGet:   std::future::Future<Output = ADSummary> + Send + 'static,
    // repo-update callback (could also be async if you need)
    FUpdate:  Fn(ADRepo) + Send + Sync + 'static,
{
    loop {
        let mut progressed_this_tick = false;

        // Walk every step once per tick.
        // Each branch must hold its lock for the **shortest** time possible.
        for step in &plan {
            // ‼️ lock for this step
            let mut guard = step.lock().await;

            match &mut *guard {
                //-----------------------------------------------------------------
                // 1️⃣  AwaitingPrompt  ➜  PendingExecution
                //-----------------------------------------------------------------
                ExecutionStep::AwaitingPrompt(sections) => {
                    let mut unresolved = 0;

                    for section in sections.iter_mut() {
                        if let PromptSection::Placeholder(ref target) = section {
                            let target_guard = target.lock().await;
                            match &*target_guard {
                                ExecutionStep::PendingSave(sum)
                                | ExecutionStep::Done(sum) => {
                                    *section = PromptSection::StaticString(sum.0.clone());
                                }
                                _ => unresolved += 1,
                            }
                        }
                    }

                    if unresolved == 0 {
                        let prompt = join_prompt_sections(sections);
                        *guard = ExecutionStep::PendingExecution(prompt);
                        progressed_this_tick = true;
                    }
                }

                //-----------------------------------------------------------------
                // 2️⃣  PendingExecution  ➜  Executing  (spawn LLM call)
                //-----------------------------------------------------------------
                ExecutionStep::PendingExecution(prompt) => {
                    let prompt_clone = prompt.clone();
                    *guard = ExecutionStep::Executing(prompt.clone());
                    progressed_this_tick = true;

                    // Clone what the async task needs *before* we move out
                    let step_clone   = Arc::clone(step);
                    let on_update    = &on_update;
                    let get_summary  = &get_summary;

                    // Spawn a detached task – it will update the step later
                    tokio::spawn(async move {
                        let summary = get_summary(prompt_clone.0).await;

                        // 2a) Mark the step as PendingSave
                        {
                            let mut s = step_clone.lock().await;
                            *s = ExecutionStep::PendingSave(summary.clone());
                        }

                        // 2b) Persist and finally mark Done
                        // (You could do actual I/O here; for demo just flip state)
                        {
                            let mut s = step_clone.lock().await;
                            *s = ExecutionStep::Done(summary.clone());
                        }

                        // 2c) Notify caller
                        on_update(dummy_repo(summary));
                    });
                }

                //-----------------------------------------------------------------
                // 3️⃣  Anything else – nothing to do inside main loop
                //-----------------------------------------------------------------
                _ => {}
            }
        }

        // ── exit when every step is Done ────────────────────────────────────────
        if plan.iter()
               .all(|s| matches!(*s.blocking_lock(), ExecutionStep::Done(_)))
        {
            break;
        }

        // If no state changes we pause briefly
        if !progressed_this_tick {
            sleep(Duration::from_millis(100)).await;
        }
    }
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
    ADPrompt(combined)
}

fn on_repo_update(repo: ADRepo) -> () {
    todo!("Implement a function to render the new repo")
}
