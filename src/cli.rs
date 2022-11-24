use bpaf::Bpaf;
use chem_eq::Equation;

/// Balance a chemical equation.
///
/// Run without args to open a tui.
#[derive(Debug, Clone, Bpaf)]
#[bpaf(version, options)]
pub struct ChemArgs {
    #[bpaf(positional, optional)]
    pub equation: Option<Equation>,
}
