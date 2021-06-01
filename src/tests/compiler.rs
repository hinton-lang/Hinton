use crate::{compiler::Compiler, parser::Parser};

#[test]
fn base_func_has_no_arity() {
    let program = match Parser::parse("") {
        Ok(ast) => ast,
        Err(_) => panic!("Parser Had Errors."),
    };

    match Compiler::compile_file("test", &program) {
        Ok(res) => {
            if res.min_arity != 0u8 && res.max_arity != 0u8 {
                panic!("Base function in script should have 0 parameters.")
            }
        }
        Err(_) => panic!("Compiler Had Errors."),
    }
}

#[test]
fn base_func_has_no_defaults() {
    let program = match Parser::parse("") {
        Ok(ast) => ast,
        Err(_) => panic!("Parser Had Errors."),
    };

    match Compiler::compile_file("test", &program) {
        Ok(res) => {
            if res.defaults.len() != 0 {
                panic!("Base function in script should have 0 default parameters.")
            }
        }
        Err(_) => panic!("Compiler Had Errors."),
    }
}

#[test]
fn test_const_pool_no_duplicate_items() {
    let src = "8.9;".repeat(500);

    let program = match Parser::parse(src.as_str()) {
        Ok(ast) => ast,
        Err(_) => panic!("Parser Had Errors."),
    };

    match Compiler::compile_file("test", &program) {
        Ok(res) => {
            if res.chunk.get_pool_size() != 1 {
                panic!("Items in the constant pool should not be duplicated.")
            }
        }
        Err(_) => panic!("Compiler Had Errors."),
    }
}


