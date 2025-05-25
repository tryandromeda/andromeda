use std::{
    any::Any,
    borrow::BorrowMut,
    cell::RefCell,
    collections::VecDeque,
    str,
    sync::{atomic::Ordering, mpsc::Receiver},
};

use nova_vm::{
    ecmascript::{
        builtins::promise_objects::promise_abstract_operations::promise_capability_records::PromiseCapability,
        execution::{
            Agent, JsResult,
            agent::{GcAgent, HostHooks, Job, Options, RealmRoot},
        },
        scripts_and_modules::script::{parse_script, script_evaluation},
        types::{self, Object, Value},
    },
    engine::context::{Bindable, GcScope},
};

use crate::{Extension, HostData, MacroTask, exit_with_parse_errors};

pub struct RuntimeHostHooks<UserMacroTask> {
    pub(crate) promise_job_queue: RefCell<VecDeque<Job>>,
    pub(crate) host_data: HostData<UserMacroTask>,
}

impl<UserMacroTask> std::fmt::Debug for RuntimeHostHooks<UserMacroTask> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime").finish()
    }
}

impl<UserMacroTask> RuntimeHostHooks<UserMacroTask> {
    pub fn new(host_data: HostData<UserMacroTask>) -> Self {
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

impl<UserMacroTask: 'static> HostHooks for RuntimeHostHooks<UserMacroTask> {
    fn enqueue_promise_job(&self, job: Job) {
        self.promise_job_queue.borrow_mut().push_back(job);
    }

    fn get_host_data(&self) -> &dyn Any {
        &self.host_data
    }
}

pub type EventLoopHandler<UserMacroTask> = fn(
    macro_task: UserMacroTask,
    agent: &mut GcAgent,
    realm_root: &RealmRoot,
    host_data: &HostData<UserMacroTask>,
);

pub struct RuntimeConfig<UserMacroTask: 'static> {
    /// Disable or not strict mode.
    pub no_strict: bool,
    /// List of js files to load.
    pub paths: (Vec<String>, Vec<&'static [u8]>),
    /// Enable or not verbose outputs.
    pub verbose: bool,
    /// Collection of Rust Extensions
    pub extensions: Vec<Extension>,
    /// Collection of builtin js sources
    pub builtins: Vec<&'static str>,
    /// User event loop handler.
    pub eventloop_handler: EventLoopHandler<UserMacroTask>,
    /// Macro tasks eventloop receiver.
    pub macro_task_rx: Receiver<MacroTask<UserMacroTask>>,
}

pub struct Runtime<UserMacroTask: 'static> {
    pub config: RuntimeConfig<UserMacroTask>,
    pub agent: GcAgent,
    pub realm_root: RealmRoot,
    pub host_hooks: &'static RuntimeHostHooks<UserMacroTask>,
}

impl<UserMacroTask> Runtime<UserMacroTask> {
    /// Create a new [Runtime] given a [RuntimeConfig]. Use [Runtime::run] to run it.
    pub fn new(
        mut config: RuntimeConfig<UserMacroTask>,
        host_data: HostData<UserMacroTask>,
    ) -> Self {
        let host_hooks = RuntimeHostHooks::new(host_data);

        let host_hooks: &RuntimeHostHooks<UserMacroTask> = &*Box::leak(Box::new(host_hooks));
        let mut agent = GcAgent::new(
            Options {
                disable_gc: false,
                print_internals: config.verbose,
            },
            host_hooks,
        );
        let create_global_object: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> =
            None;
        let create_global_this_value: Option<
            for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>,
        > = None;
        let realm_root = agent.create_realm(
            create_global_object,
            create_global_this_value,
            Some(
                |agent: &mut Agent, global_object: Object<'_>, mut gc: GcScope<'_, '_>| {
                    for extension in &mut config.extensions {
                        extension.load::<UserMacroTask>(agent, global_object, gc.borrow_mut())
                    }
                },
            ),
        );

        Self {
            config,
            agent,
            realm_root,
            host_hooks,
        }
    }

    /// Run the Runtime with the specified configuration.
    pub fn run(mut self) -> RuntimeOutput {
        // Load the builtins js sources
        self.agent.run_in_realm(&self.realm_root, |agent, mut gc| {
            for builtin in &self.config.builtins {
                let realm = agent.current_realm(gc.nogc());
                let source_text = types::String::from_str(agent, builtin, gc.nogc());
                let script = match parse_script(
                    agent,
                    source_text,
                    realm,
                    !self.config.no_strict,
                    None,
                    gc.nogc(),
                ) {
                    Ok(script) => script,
                    Err(diagnostics) => exit_with_parse_errors(diagnostics, "<runtime>", builtin),
                };
                match script_evaluation(agent, script.unbind(), gc.reborrow()) {
                    Ok(_) => (),
                    Err(_) => println!("Error in runtime"),
                }
            }
        });

        let mut result = JsResult::Ok(Value::Null);

        // Fetch the runtime mod.ts file using a macro and add it to the paths
        for path in &self.config.paths.0 {
            let file = std::fs::read_to_string(path).unwrap();

            result = self.agent.run_in_realm(&self.realm_root, |agent, mut gc| {
                let source_text = types::String::from_string(agent, file, gc.nogc());
                let realm = agent.current_realm(gc.nogc());

                let script = match parse_script(
                    agent,
                    source_text,
                    realm,
                    !self.config.no_strict,
                    None,
                    gc.nogc(),
                ) {
                    Ok(script) => script,
                    Err(errors) => exit_with_parse_errors(errors, path, source_text.as_str(agent)),
                };

                script_evaluation(agent, script.unbind(), gc.reborrow()).unbind()
            });
        }

        for code in &self.config.paths.1 {
            let source_text = String::from_utf8_lossy(code).into_owned();
            result = self.agent.run_in_realm(&self.realm_root, |agent, mut gc| {
                let source_text = types::String::from_string(agent, source_text, gc.nogc());
                let realm = agent.current_realm(gc.nogc());

                let script = match parse_script(
                    agent,
                    source_text,
                    realm,
                    !self.config.no_strict,
                    None,
                    gc.nogc(),
                ) {
                    Ok(script) => script,
                    Err(errors) => {
                        exit_with_parse_errors(errors, "internal", source_text.as_str(agent))
                    }
                };

                script_evaluation(agent, script.unbind(), gc.reborrow()).unbind()
            });
        }

        loop {
            while let Some(job) = self.host_hooks.pop_promise_job() {
                result = self.agent.run_in_realm(&self.realm_root, |agent, mut gc| {
                    job.run(agent, gc.reborrow()).unbind().map(|_| Value::Null)
                });
            }

            // If both the microtasks and macrotasks queues are empty we can end the event loop
            if !self.host_hooks.any_pending_macro_tasks() {
                break;
            }

            self.handle_macro_task();
        }

        RuntimeOutput {
            agent: self.agent,
            realm_root: self.realm_root,
            result,
        }
    }

    // Listen for pending macro tasks and resolve one by one
    pub fn handle_macro_task(&mut self) {
        match self.config.macro_task_rx.recv() {
            Ok(MacroTask::ResolvePromise(root_value)) => {
                self.agent.run_in_realm(&self.realm_root, |agent, gc| {
                    let value = root_value.take(agent);
                    if let Value::Promise(promise) = value {
                        let promise_capability = PromiseCapability::from_promise(promise, false);
                        promise_capability.resolve(agent, Value::Undefined, gc);
                    } else {
                        panic!("Attempted to resolve a non-promise value");
                    }
                });
            }
            // Let the user runtime handle its macro tasks
            Ok(MacroTask::User(e)) => {
                (self.config.eventloop_handler)(
                    e,
                    &mut self.agent,
                    &self.realm_root,
                    &self.host_hooks.host_data,
                );
            }
            _ => {}
        }
    }
}

pub struct RuntimeOutput {
    pub agent: GcAgent,
    pub realm_root: RealmRoot,
    pub result: JsResult<'static, Value<'static>>,
}
