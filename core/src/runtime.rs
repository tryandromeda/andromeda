use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    sync::{atomic::Ordering, mpsc::Receiver},
};

use nova_vm::ecmascript::{
    builtins::promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability,
    execution::{
        agent::{GcAgent, HostHooks, Job, Options, RealmRoot},
        Agent, JsResult,
    },
    scripts_and_modules::script::{parse_script, script_evaluation},
    types::{self, Object, Value},
};

use crate::{
    exit_with_parse_errors, initialize_recommended_builtins, initialize_recommended_extensions,
    HostData, MacroTask,
};

pub struct RuntimeHostHooks {
    pub(crate) promise_job_queue: RefCell<VecDeque<Job>>,
    pub(crate) host_data: HostData,
}

impl std::fmt::Debug for RuntimeHostHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime").finish()
    }
}

impl RuntimeHostHooks {
    pub fn new(host_data: HostData) -> Self {
        Self {
            promise_job_queue: RefCell::default(),
            host_data,
        }
    }

    pub fn pop_promise_job(&self) -> Option<Job> {
        self.promise_job_queue.borrow_mut().pop_front()
    }

    pub fn any_pending_macro_tasks(&self) -> bool {
        self.host_data.macro_task_count.load(Ordering::Relaxed) > 0
    }
}

impl HostHooks for RuntimeHostHooks {
    fn enqueue_promise_job(&self, job: Job) {
        self.promise_job_queue.borrow_mut().push_back(job);
    }

    fn get_host_data(&self) -> &dyn Any {
        &self.host_data
    }
}

pub struct RuntimeConfig {
    pub no_strict: bool,
    pub paths: Vec<String>,
    pub verbose: bool,
}

pub struct Runtime {
    pub config: RuntimeConfig,
    pub agent: GcAgent,
    pub realm_root: RealmRoot,
    pub host_hooks: &'static RuntimeHostHooks,
    pub macro_task_rx: Receiver<MacroTask>,
}

impl Runtime {
    /// Create a new [Runtime] given a [RuntimeConfig]. Use [Runtime::run] to run it.
    pub fn new(config: RuntimeConfig) -> Self {
        let (host_data, macro_task_rx) = HostData::new();
        let host_hooks = RuntimeHostHooks::new(host_data);

        let host_hooks: &RuntimeHostHooks = &*Box::leak(Box::new(host_hooks));
        let mut agent = GcAgent::new(
            Options {
                disable_gc: false,
                print_internals: config.verbose,
            },
            host_hooks,
        );
        let create_global_object: Option<fn(&mut Agent) -> Object> = None;
        let create_global_this_value: Option<fn(&mut Agent) -> Object> = None;
        let realm_root = agent.create_realm(
            create_global_object,
            create_global_this_value,
            Some(initialize_recommended_extensions),
        );

        Self {
            config,
            agent,
            realm_root,
            host_hooks,
            macro_task_rx,
        }
    }

    /// Run the Runtime with the specified configuration.
    pub fn run(&mut self) -> JsResult<Value> {
        // Load the builtins js sources
        self.agent.run_in_realm(&self.realm_root, |agent| {
            initialize_recommended_builtins(agent, self.config.no_strict);
        });

        let mut final_result = Value::Null;

        // Fetch the runtime mod.ts file using a macro and add it to the paths
        for path in &self.config.paths {
            let file = std::fs::read_to_string(path).unwrap();

            final_result = self.agent.run_in_realm(&self.realm_root, |agent| {
                let source_text = types::String::from_string(agent, file);
                let realm = agent.current_realm_id();

                let script =
                    match parse_script(agent, source_text, realm, !self.config.no_strict, None) {
                        Ok(script) => script,
                        Err(errors) => {
                            exit_with_parse_errors(errors, path, source_text.as_str(agent))
                        }
                    };

                script_evaluation(agent, script)
            })?;
        }

        loop {
            while let Some(job) = self.host_hooks.pop_promise_job() {
                self.agent
                    .run_in_realm(&self.realm_root, |agent| job.run(agent))?;
            }

            // If both the microtasks and macrotasks queues are empty we can end the event loop
            if !self.host_hooks.any_pending_macro_tasks() {
                break;
            }

            self.handle_macro_task();
        }

        Ok(final_result)
    }

    // Listen for pending macro tasks and resolve one by one
    pub fn handle_macro_task(&mut self) {
        #[allow(clippy::single_match)]
        match self.macro_task_rx.recv() {
            Ok(MacroTask::ResolvePromise(root_value)) => {
                self.agent.run_in_realm(&self.realm_root, |agent| {
                    let value = root_value.take(agent);
                    if let Value::Promise(promise) = value {
                        let promise_capability = PromiseCapability::from_promise(promise, false);
                        promise_capability.resolve(agent, Value::Undefined);
                    } else {
                        panic!("Attempted to resolve a non-promise value");
                    }
                });
            }
            Ok(MacroTask::RunInterval(interval_id)) => interval_id.run(
                &mut self.agent,
                &self.host_hooks.host_data,
                &self.realm_root,
            ),
            Ok(MacroTask::ClearInterval(interval_id)) => {
                interval_id.clear_and_abort(&self.host_hooks.host_data);
            }
            Ok(MacroTask::RunAndClearTimeout(timeout_id)) => timeout_id.run_and_clear(
                &mut self.agent,
                &self.host_hooks.host_data,
                &self.realm_root,
            ),
            Ok(MacroTask::ClearTimeout(timeout_id)) => {
                timeout_id.clear_and_abort(&self.host_hooks.host_data);
            }
            _ => {}
        }
    }
}
