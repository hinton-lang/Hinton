// ****************************
// The contents of this file are experimental.
// ****************************

use std::rc::Rc;

use crate::{
    intermediate::ast::{ASTNode, BinaryExprNode, BinaryExprType, ModuleNode},
    objects::Object,
};

pub fn analyze_module<'a>(program: Rc<ModuleNode<'a>>) {
    for node in program.body.iter() {
        // TODO: What can we do so that cloning each node is no longer necessary?
        // Cloning each node is a very expensive operation because some of the nodes
        // could have an arbitrarily big amount of data. Fox example, large bodies
        // of literal text could drastically slow down the performance of the compiler
        // when those strings have to be cloned.
        analyze_node(node.clone());
    }
}

fn analyze_node<'a>(node: ASTNode<'a>) {
    return match node {
        ASTNode::Binary(x) => check_valid_division(x),
        ASTNode::PrintStmt(x) => analyze_node(*x.child),
        ASTNode::ExpressionStmt(x) => analyze_node(*x.child),
        _ => {}
    };
}

fn check_valid_division<'a>(node: BinaryExprNode<'a>) {
    if let BinaryExprType::Division = node.opr_type {
        let rhs = get_object_value(*node.right);

        match *rhs {
            Object::Number(x) if x != 0f64 => {
                return;
            }
            Object::Number(x) if x == 0f64 => {
                println!("**StaticAnalysis Error**: Cannot divide by zero.");
            }
            _ => println!("**StaticAnalysis Error**: Cannot divide by object of type: '{}'.", rhs.type_name()),
        }
    }
}

fn get_object_value<'a>(node: ASTNode<'a>) -> Rc<Object<'a>> {
    return match node {
        ASTNode::Literal(x) => x.value,
        _ => get_object_value(node),
    };
}
