use std::{
    fmt::{self, Display},
    usize,
};

/// Types of symbols available in Hinton.
#[derive(Clone)]
pub enum SymbolType {
    Variable,
    Constant,
    Function,
    Class,
    Enum,
    Parameter,
}

/// Represents a symbol found in a particular scope.
pub enum SymbolLoc {
    /// Represents the symbol and stack position
    /// of a function's local declaration.
    Local(Symbol, usize),
    /// Represents the symbol of a global declaration,
    /// and the pool position of the symbol's name.
    Global(Symbol, usize),
    /// Represents a native function symbol.
    Native,
    /// Represents a not-found symbol.
    None,
}

/// Represents a symbol. Used for lexical scoping.
#[derive(Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_depth: usize,
    pub symbol_type: SymbolType,
    pub is_initialized: bool,
    pub is_used: bool,
    pub line_info: (usize, usize),
    pub is_global: bool,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = format!(
            "(name: {}, scope: {})",
            self.name.as_str(),
            self.symbol_depth
        );
        fmt::Debug::fmt(s.as_str(), f)
    }
}

/// Represents the list of local symbols in a particular function.
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn push(&mut self, symbol: Symbol) {
        self.symbols.push(symbol)
    }

    pub fn pop(&mut self) -> Option<Symbol> {
        self.symbols.pop()
    }

    pub fn get(&self, pos: usize) -> Option<&Symbol> {
        self.symbols.get(pos)
    }

    pub fn find_in_scope(&self, name: &String, scope: usize) -> Option<&Symbol> {
        for symbol in self.symbols.iter().rev() {
            // Only look for the symbol in the current scope.
            if symbol.symbol_depth < scope {
                break;
            }

            if &symbol.name == name {
                return Some(symbol);
            }
        }

        None
    }

    pub fn resolve(&mut self, name: &String, mark_used: bool) -> Option<(Symbol, usize)> {
        for (idx, symbol) in self.symbols.iter_mut().enumerate().rev() {
            if &symbol.name == name {
                if mark_used {
                    symbol.is_used = true;
                }
                return Some((symbol.clone(), idx));
            }
        }

        None
    }

    pub fn pop_scope(
        &mut self,
        min_depth: usize,
        pop_symbols: bool,
        show_warning: bool,
    ) -> (usize, (usize, usize)) {
        let mut pop_count = 0usize;
        let mut last_symbol_pos = (0, 0);

        // We get the ith symbol (from the back) instead of getting
        // the `.last()` because when the `pop_symbol` parameter is
        // false, the loop may become infinite (because we are not
        // popping the symbol off the table)
        let mut ith = self.len();

        while let Some(symbol) = self.symbols.get(ith - 1) {
            if symbol.symbol_depth >= min_depth {
                let is_used = symbol.is_used;
                let symbol_name = symbol.name.clone();

                last_symbol_pos = symbol.line_info;
                pop_count += 1;

                // Because variables live in the stack, once we are done with
                // them for this scope, we take them out of the stack by emitting
                // the OP_POP_STACK instruction for each one of the variables.
                if pop_symbols {
                    self.symbols.pop().unwrap();
                }

                if !is_used && show_warning {
                    println!(
                        "\x1b[33;1mWarning\x1b[0m at [{}:{}] – Variable '\x1b[1m{}\x1b[0m' is never used.",
                        last_symbol_pos.0, last_symbol_pos.1, symbol_name
                    );
                }

                ith -= 1;
                continue;
            }

            break;
        }

        (pop_count, last_symbol_pos)
    }
}
