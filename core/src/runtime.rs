use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    sync::{atomic::Ordering, mpsc::Receiver},
};

use nova_vm::ecmascript::{
    builtins::promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability,
    execution::{
        agent::{HostHooks, Job, Options},
        initialize_host_defined_realm, Agent, JsResult, Realm,
    },
    scripts_and_modules::script::{parse_script, script_evaluation},
    types::{Object, Value},
};
use oxc_allocator::Allocator;

use crate::{
    exit_with_parse_errors, initialize_recommended_builtins, initialize_recommended_extensions,
    HostData, MacroTask,
};

pub struct RuntimeHostHooks {
    promise_job_queue: RefCell<VecDeque<Job>>,
    host_data: HostData,
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
    pub db_path: String,
    pub verbose: bool,
}

pub struct Runtime {
    pub config: RuntimeConfig,
    pub agent: Agent,
    pub allocator: Allocator,
    pub host_hooks: &'static RuntimeHostHooks,
    pub macro_task_rx: Receiver<MacroTask>,
}

impl Runtime {
    /// Create a new [Runtime] given a [RuntimeConfig]. Use [Runtime::run] to run it.
    pub fn new(config: RuntimeConfig) -> Self {
        let (host_data, macro_task_rx) = HostData::new();
        let host_hooks = RuntimeHostHooks::new(host_data);

        let host_hooks: &RuntimeHostHooks = &*Box::leak(Box::new(host_hooks));
        let mut agent = Agent::new(
            Options {
                disable_gc: false,
                print_internals: config.verbose,
            },
            host_hooks,
        );
        {
            let create_global_object: Option<fn(&mut Realm) -> Object> = None;
            let create_global_this_value: Option<fn(&mut Realm) -> Object> = None;
            initialize_host_defined_realm(
                &mut agent,
                create_global_object,
                create_global_this_value,
                Some(initialize_recommended_extensions),
            );
        }
        let allocator = Allocator::default();

        Self {
            config,
            allocator,
            agent,
            host_hooks,
            macro_task_rx,
        }
    }

    /// Run the Runtime with the specified configuration.
    pub fn run(&mut self) -> JsResult<Value> {
        let realm = self.agent.current_realm_id();

        // Load the builtins js sources
        initialize_recommended_builtins(&self.allocator, &mut self.agent, self.config.no_strict);

        let mut final_result = Value::Null;

        // Fetch the runtime mod.ts file using a macro and add it to the paths
        for path in &self.config.paths {
            let file = std::fs::read_to_string(path).unwrap();
            let script = match parse_script(
                &self.allocator,
                file.into(),
                realm,
                !self.config.no_strict,
                None,
            ) {
                Ok(script) => script,
                Err((file, errors)) => exit_with_parse_errors(errors, path, &file),
            };
            final_result = match script_evaluation(&mut self.agent, script) {
                Ok(v) => v,
                err @ _ => return err,
            }
        }

        loop {
            while let Some(job) = self.host_hooks.pop_promise_job() {
                if let Err(err) = job.run(&mut self.agent) {
                    return Err(err);
                }
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
                let value = root_value.take(&mut self.agent);
                if let Value::Promise(promise) = value {
                    let promise_capability = PromiseCapability::from_promise(promise, false);
                    promise_capability.resolve(&mut self.agent, Value::Undefined);
                } else {
                    panic!("Attempted to resolve a non-promise value");
                }
            }
            _ => {}
        }
    }
}
