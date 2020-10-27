package org.hinton_lang.Helper;

import org.hinton_lang.AbstractSyntaxTree.*;

public class PrintAST {
    private AST left;
    private AST right;
    private String name;

    private final String RESET = "\u001B[0m";
    private final String RED = "\u001B[38;5;1m";
    private final String SOFT_RED = "\u001B[38;5;166m";
    private final String PINK = "\u001B[38;5;210m";
    private final String GREEN = "\u001B[38;5;35m";
    private final String GOLD = "\u001B[38;5;220m";
    private final String BLUE = "\u001B[38;5;81m";
    private final String SOFT_GREEN = "\u001B[38;5;66m";
    private final String DEEP_GREEN = "\u001B[38;5;70m";
    private final String DEEP_GREEN2 = "\u001B[38;5;71m";
    private final String DEEP_OCEAN = "\u001B[38;5;31m";
    private final String WHITE = "\u001B[38;5;255m";
    private final String GREY = "\u001B[38;5;245m";
    private final String PURPLE = "\u001B[38;5;98m";
    private final String SOFT_PURPLE = "\u001B[38;5;146m";
    private final String ORANGE = "\u001B[38;5;214m";
    private final String DEEP_ORANGE = "\u001B[38;5;166m";
    private final String PALE_ORANGE = "\u001B[38;5;187m";

    
    public String print(AST tree) {
        StringBuilder sb = new StringBuilder();
        if (tree instanceof Program) {
            sb.append("\nPROGRAM\n");
            for (AST ast : ((Program)tree).statements) sb.append("├───" + print(ast) + "│\n");
        } else {
            traversePreOrder(sb, "", "", tree);
        }
        return sb.toString();
    }
    
    public void traversePreOrder(StringBuilder sb, String padding, String pointer, AST node) {
        if (node != null) {
            left = node.left;
            right = node.right;
            name = RED + "NOT RECOGNIZED";

            // For this recursion to work, the node must be casted to a specific
            // type before it is fed into the next iteration of the recursion.
            if (node instanceof Assignment) {
                left = ((Assignment)node).left;
                right = ((Assignment)node).right;
                name = SOFT_GREEN + "Assignment (" + ((Assignment)node).type + ")";
            } else if (node instanceof ReAssignment) {
                left = ((ReAssignment)node).left;
                right = ((ReAssignment)node).right;
                name = SOFT_GREEN + "Re-Assignment (" + ((ReAssignment)node).type + ")";
            } else if (node instanceof Compound) {
                name = "Compound";
                String p = doTraversal(sb, padding, pointer, null, null, name);
                for (AST ast : ((Compound)node).children) traversePreOrder(sb, padding, p, ast);
            } else if (node instanceof VarDeclaration) {
                left = ((VarDeclaration)node).left;
                right = ((VarDeclaration)node).right;
                name = ORANGE + "VarDeclaration";
            } else if (node instanceof ConstDeclaration) {
                left = ((ConstDeclaration)node).left;
                right = ((ConstDeclaration)node).right;
                name = DEEP_ORANGE + "ConstDeclaration";
            } else if (node instanceof FuncDeclaration) {
                left = ((FuncDeclaration)node).left;
                right = ((FuncDeclaration)node).right;
                name = DEEP_ORANGE + "FuncDeclaration";
            } else if (node instanceof TypeDef) {
                name = PALE_ORANGE + "TypeDef: " + GOLD + ((TypeDef)node).type;
            } else if (node instanceof BinaryOperator) {
                left = ((BinaryOperator)node).left;
                right = ((BinaryOperator)node).right;
                name = PURPLE + "BinaryOperator: " + SOFT_PURPLE + ((BinaryOperator)node).operator.text;
            } else if (node instanceof BoolLiteral) {
                name = "BoolLiteral: " + ((BoolLiteral)node).token.text;
            } else if (node instanceof Identifier) {
                name = DEEP_OCEAN + "Identifier: " + SOFT_GREEN + ((Identifier)node).token.text;
            } else if (node instanceof IntLiteral) {
                name = SOFT_RED + "IntLiteral: " + PINK + ((IntLiteral)node).value;
            } else if (node instanceof RealLiteral) {
                name = "RealLiteral: " + ((RealLiteral)node).value;
            } else if (node instanceof StringLiteral) {
                name = DEEP_GREEN + "StringLiteral: " + DEEP_GREEN2 + ((StringLiteral)node).token.text;
            } else if (node instanceof UnaryOperator) {
                left = ((UnaryOperator)node).left;
                right = ((UnaryOperator)node).right;
                name = "UnaryOperator";
            } else if (node instanceof NoOp) {
                name = GREY + "NO-OPERATOR";
            }


            // Goes to the next recursion.
            if (!(node instanceof Compound)) doTraversal(sb, padding, pointer, left, right, name);
        }
    }


    private String doTraversal(StringBuilder sb, String padding, String pointer, AST left, AST right, String name) {
        sb.append(padding);
        sb.append(pointer);
        sb.append(" " + name + RESET);
        sb.append("\n");

        StringBuilder paddingBuilder = new StringBuilder(padding);
        paddingBuilder.append("│  ");

        String paddingForBoth = paddingBuilder.toString();
        String pointerRight = "└───";
        String pointerLeft = (right != null) ? "├───" : "└───";

        traversePreOrder(sb, paddingForBoth, pointerLeft, left);
        traversePreOrder(sb, paddingForBoth, pointerRight, right);

        return paddingForBoth;
    }
}
