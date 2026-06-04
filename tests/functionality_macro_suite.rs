use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use cmdkit::{CMDKit, Command};
use cmdkit_macros::strategy;

#[strategy]
fn simple_cli_strategy(
    _ctx: cmdkit::ExecutionContext,
    _args: cmdkit::InvocationArgs,
) -> Result<(), cmdkit::StrategyError> {
    Ok(())
}

#[test]
fn cli_attribute_generates_execute_shaped_strategy_wrapper() {
    let core = CMDKit::builder()
        .register(Command::new(
            "simple",
            "simple cli strategy",
            SimpleCliStrategy::new(),
        ))
        .build();

    let args = vec!["app".to_string(), "simple".to_string()];
    assert!(core.try_run_from_args(&args).is_ok());
}

#[test]
fn strategy_attribute_generated_type_uses_upper_camel_name() {
    let _instance = SimpleCliStrategy::new();
}

#[strategy]
fn create_directory(
    _ctx: cmdkit::ExecutionContext,
    args: cmdkit::InvocationArgs,
) -> Result<(), cmdkit::StrategyError> {
    assert!(!args.name.is_empty());
    Ok(())
}

#[test]
fn test_command_suit() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let dir_path: PathBuf = env::temp_dir().join(format!("cli-core-create-{unique}"));

    let core = CMDKit::builder()
        .register(Command::new(
            "create",
            "Create a directory",
            CreateDirectory::new(),
        ))
        .build();

    let args = vec!["app".to_string(), "create".to_string()];

    assert!(core.try_run_from_args(&args).is_ok());
    assert!(!dir_path.exists());
}
