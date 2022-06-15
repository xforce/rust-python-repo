fn run() -> anyhow::Result<()> {
    use rustpython_vm as vm;

    vm::Interpreter::with_init(Default::default(), |vm| {
        vm.add_native_modules(rustpython_stdlib::get_module_inits());
        vm.add_frozen(rustpython_vm::py_freeze!(dir = "scripts/modules"));
    })
    .enter(|vm| {
        let scope = vm.new_scope_with_builtins();

        let code_obj = vm
            .compile(
                r#"from nested1.nested2 import meow"#,
                vm::compile::Mode::Exec,
                "<embedded>".to_owned(),
            )
            .map_err(|err| vm.new_syntax_error(&err))
            .unwrap();

        if let Err(e) = vm.run_code_obj(code_obj, scope.clone()) {
            vm.print_exception(e.clone());
        } else {
            eprintln!("No Error");
        }

        Ok(())
    })
}

fn main() -> anyhow::Result<()> {
    use tracing_subscriber::{filter::EnvFilter, FmtSubscriber};

    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();
    let filter = filter.add_directive("rustpython_vm::frame=error".parse().unwrap());

    let subscriber = if cfg!(debug_assertions) {
        FmtSubscriber::builder().with_max_level(tracing::Level::DEBUG)
    } else {
        FmtSubscriber::builder().with_max_level(tracing::Level::INFO)
    }
    .with_env_filter(filter);
    let _ = subscriber.try_init();

    let child = std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(run)
        .unwrap();

    child.join().unwrap()
}
