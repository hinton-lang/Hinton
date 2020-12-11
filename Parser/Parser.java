package org.hinton_lang.Parser;

import java.util.List;

import org.hinton_lang.Tokens.*;
import static org.hinton_lang.Tokens.TokenType.*;
import org.hinton_lang.Parser.AST.Expr;
import org.hinton_lang.Hinton;

public class Parser {
    private static class ParseError extends RuntimeException {
    }

    private final List<Token> tokens;
    private int current = 0;

    public Parser(List<Token> tokens) {
        this.tokens = tokens;
    }

    public Expr parse() {
        try {
            return expression();
        } catch (ParseError error) {
            return null;
        }
    }

    private boolean match(TokenType... types) {
        for (TokenType type : types) {
            if (check(type)) {
                advance();
                return true;
            }
        }

        return false;
    }

    private Token consume(TokenType type, String message) {
        if (check(type))
            return advance();

        throw error(peek(), message);
    }

    private boolean check(TokenType type) {
        if (isAtEnd())
            return false;
        return peek().type == type;
    }

    private Token advance() {
        if (!isAtEnd())
            current++;
        return previous();
    }

    private boolean isAtEnd() {
        return peek().type == END_OF_FILE;
    }

    private Token peek() {
        return tokens.get(current);
    }

    private Token previous() {
        return tokens.get(current - 1);
    }

    private ParseError error(Token token, String message) {
        Hinton.error(token, message);
        return new ParseError();
    }

    private void synchronize() {
        advance();

        while (!isAtEnd()) {
            if (previous().type == SEMICOLON_SEPARATOR)
                return;

            switch (peek().type) {
                case CLASS_KEYWORD:
                case FUNC_KEYWORD:
                case LET_KEYWORD:
                case CONST_KEYWORD:
                case FOR_KEYWORD:
                case WHILE_KEYWORD:
                case IF_KEYWORD:
                    // case PRINT:
                case RETURN_KEYWORD:
                    return;
            }

            advance();
        }
    }

    private Expr expression() {
        return equality();
    }

    private Expr equality() {
        Expr expr = comparison();

        while (match(LOGICAL_NOT_EQ, LOGICAL_EQ)) {
            Token operator = previous();
            Expr right = comparison();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    private Expr comparison() {
        Expr term = term();

        while (match(LESS_THAN, LESS_THAN_EQ, GREATER_THAN, GREATER_THAN_EQ)) {
            Token operator = previous();
            Expr right = term();
            term = new Expr.Binary(term, operator, right);
        }

        return term;
    }

    private Expr term() {
        Expr expr = factor();

        while (match(MINUS, PLUS)) {
            Token operator = previous();
            Expr right = factor();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    private Expr factor() {
        Expr expr = expo();

        while (match(DIV, MULT, MOD)) {
            Token operator = previous();
            Expr right = expo();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    private Expr expo() {
        Expr expr = unary();

        while (match(EXPO)) {
            Token operator = previous();
            Expr right = unary();
            return new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    private Expr unary() {
        if (match(LOGICAL_NOT, MINUS, PLUS)) {
            Token operator = previous();
            Expr right = unary();
            return new Expr.Unary(operator, right);
        } else {
            return primary();
        }
    }

    private Expr primary() {
        if (match(BOOL_LITERAL_FALSE))
            return new Expr.Literal(false);
        if (match(BOOL_LITERAL_TRUE))
            return new Expr.Literal(true);
        if (match(NULL_LITERAL))
            return new Expr.Literal(null);

        if (match(INTEGER_LITERAL, REAL_LITERAL, STRING_LITERAL)) {
            return new Expr.Literal(previous().literal);
        }

        if (match(L_PARENTHESIS)) {
            Expr expr = expression();
            consume(R_PARENTHESIS, "Expected ')' after expression.");
            return new Expr.Grouping(expr);
        }

        throw error(peek(), "Expect expression.");
    }
}