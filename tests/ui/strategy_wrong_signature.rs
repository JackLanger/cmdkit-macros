use cmdkit_macros::strategy;

#[strategy]
fn wrong_strategy(
    _ctx: cmdkit::ExecutionContext,
    arguments: Vec<cmdkit::Argument>,
) -> Result<(), cmdkit::StrategyError> {
    let _ = arguments;
    Ok(())
}

fn main() {}
