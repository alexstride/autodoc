//Copied example


use std::sync::Arc;
use tokio::sync::Mutex;      // async-aware mutex
use tokio::time::{sleep, Duration};

pub type SharedStep = Arc<Mutex<ExecutionStep>>;
pub type ADPlan      = Vec<SharedStep>;

#[derive(Debug)]
enum PromptSection {
    StaticString(String),
    Placeholder(SharedStep),
}

#[derive(Debug)]
enum ExecutionStep {
    AwaitingPrompt(Vec<PromptSection>),
    PendingExecution(ADPrompt),   // ready to call the LLM
    Executing(ADPrompt),          // LLM call in-flight
    PendingSave(ADSummary),       // got summary, waiting to be persisted
    Done(ADSummary),
}

#[derive(Debug, Clone)]
struct ADPrompt(String);
#[derive(Debug, Clone)]
struct ADSummary(String);


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
