use chem_eq::balance::EquationBalancer;

mod cli;
mod ui;

fn main() -> color_eyre::Result<()> {
    // setup
    color_eyre::install()?;
    let args = cli::chem_args().run();

    if let Some(eq) = args.equation.as_ref() {
        println!("{}", EquationBalancer::from(eq).balance());
        return Ok(());
    }

    ui::tui()?;

    Ok(())
}
