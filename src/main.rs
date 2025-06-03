// TODO: remove!
#![allow(warnings)]


pub mod crawler;
pub mod tree_renderer;
pub mod common;
pub mod autodoc;

use clap::{Parser, ValueHint};
use std::{collections::HashMap, path::PathBuf, thread, time::Duration};
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

struct ADSummary(String);

fn load_repo(repo_root: PathBuf, config: &AutodocConfig) -> ADRepo {
    todo!("implement a function which crawls the whole repo and loads it into the ADRepo dataclass")
}

struct TreeNodePlan {
    
}
struct ADPlan<'a>(Vec<ExecutionStep<'a>>);

fn create_plan(repo: ADRepo, start_path: &PathBuf, config: &AutodocConfig) -> ADPlan {
    todo!("Implement a function which takes the repo and the start path and uses it to create an execution plan")
}

enum ExecutionStatus {
    AwaitingPrompt,
    PendingExecution,
    Executing,
    PendingSave,
    Done
}

enum PromptSection<'a> {
    StaticString(String),
    Placeholder(&'a ExecutionStep<'a>)
}
struct ExecutionStep<'a> {
    status: ExecutionStatus,
    prompt_sections: Vec<PromptSection<'a>>
}

fn execute_plan(
    plan: ADPlan,
    config: &AutodocConfig,
    on_update: fn(ADRepo) -> (),
    get_summary: fn(String) -> ADSummary
) -> () {
    let is_complete = false;
    // Assume that the ADPlan is a list of execution steps
    // Loop over all of the values in the hashmap and find the first one which is in a state which can be progressed
    while (!is_complete) {
        thread::sleep(Duration::from_millis(100));
        
    }
}

fn on_repo_update(repo: ADRepo) -> () {
    todo!("Implement a function to render the new repo")
}
