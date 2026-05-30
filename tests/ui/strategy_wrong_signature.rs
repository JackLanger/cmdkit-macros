use cmdkit_macros::strategy;

#[strategy]
fn wrong_strategy(
    options: Vec<u32>,
    arguments: Vec<cmdkit::Argument>,
    subcommands: Vec<String>,
) -> Result<(), cmdkit::StrategyError> {
    let _ = (options, arguments, subcommands);
    Ok(())
}

fn main() {}
