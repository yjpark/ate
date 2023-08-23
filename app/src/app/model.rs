use fermi::prelude::*;
use ate_model::prelude::*;

pub static ENTRIES: Atom<Vec<Entry>> = Atom(|_| vec![]);

pub static HISTORY: Atom<Vec<Entry>> = Atom(|_| vec![]);
