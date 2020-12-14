package org.hinton_lang.Parser;

import java.util.ArrayList;
import java.util.List;

// Project-specific
import org.hinton_lang.Tokens.*;
import static org.hinton_lang.Tokens.TokenType.*;
import org.hinton_lang.Parser.AST.Expr;
import org.hinton_lang.Hinton;
import org.hinton_lang.Errors.ParseError;
import org.hinton_lang.Parser.AST.Stmt;

public class Parser {
    private final List<Token> tokens;
    private int current = 0;

    public Parser(List<Token> tokens) {
        this.tokens = tokens;
    }

    /**
     * Parses the provided list of tokens to generate am Abstract Syntax Tree (AST)
     * 
     * @return An AST representation of the source code.
     */
    public List<Stmt> parse() {
        List<Stmt> statements = new ArrayList<>();
        while (!isAtEnd()) {
            statements.add(declaration());
        }

        return statements;
    }

    /**
     * Checks if the current token matches any of the provided tokens.
     * 
     * @param types The tokens to be matched against the current token
     * @return True if the current token matches any of the provided tokens, false
     *         otherwise.
     */
    private boolean match(TokenType... types) {
        for (TokenType type : types) {
            if (check(type)) {
                advance();
                return true;
            }
        }

        return false;
    }

    /**
     * Consume the current token, making sure that the current token is the token we
     * expected to consume.
     * 
     * @param type    The token we expect to consume
     * @param message A message displayed in the case that the current token is not
     *                the type we expected to consume
     * @return The consumed token
     */
    private Token consume(TokenType type, String message) {
        if (check(type))
            return advance();

        throw error(peek(), message);
    }

    /**
     * Checks that the current token matches the provided type without consuming the
     * current token.
     * 
     * @param type The type we expect to see
     * @return True if the current token matches the provided type, false otherwise.
     */
    private boolean check(TokenType type) {
        if (isAtEnd())
            return false;
        return peek().type == type;
    }

    /**
     * Advances the token pointer to the next token.
     * 
     * @return The previous token (before advancing).
     */
    private Token advance() {
        if (!isAtEnd())
            current++;
        return previous();
    }

    /**
     * Checks whether we are currently reading the last token or not.
     * 
     * @return True if we are at the last token, false otherwise
     */
    private boolean isAtEnd() {
        return peek().type == END_OF_FILE;
    }

    /**
     * Gets the current token from the list without consuming it.
     * 
     * @return The current token.
     */
    private Token peek() {
        return tokens.get(current);
    }

    /**
     * Gets the previous token from the list without consuming it.
     * 
     * @return The previous token.
     */
    private Token previous() {
        return tokens.get(current - 1);
    }

    /**
     * Reports a Parse error whe the token found was not expected.
     * 
     * @param token   The unexpected token.
     * @param message The message to display to the user.
     * @return The error.
     */
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
                default:
                    break;
            }

            advance();
        }
    }

    /**
     * Matches a statement as specified in the grammar.cfg file.
     * 
     * @return A statement.
     */
    private Stmt statement() {
        if (match(PRINT_KEYWORD))
            return printStatement();
        if (match(L_CURLY_BRACES))
            return new Stmt.Block(block());

        return expressionStatement();
    }

    /**
     * Matches a print statement as specified in the grammar.cfg file.
     * 
     * @return A print statement.
     */
    private Stmt printStatement() {
        consume(L_PARENTHESIS, "Expected '(' after \"print\" keyword.");
        Expr value = expression();
        consume(R_PARENTHESIS, "Expected ')' after expression.");
        if (check(SEMICOLON_SEPARATOR))
            consume(SEMICOLON_SEPARATOR, "Expected ';'");
        return new Stmt.Print(value);
    }

    /**
     * Matches a variable declaration as specified in the grammar.cfg file.
     * 
     * @return A variable declaration.
     */
    private Stmt varDeclaration() {
        Token name = consume(IDENTIFIER, "Expect variable name.");

        Expr initializer = null;
        if (match(EQUALS_SIGN)) {
            initializer = expression();
        }

        consume(SEMICOLON_SEPARATOR, "Expect ';' after variable declaration.");

        return new Stmt.Var(name, initializer);
    }

    /**
     * Matches a variable declaration as specified in the grammar.cfg file.
     * 
     * @return A variable declaration.
     */
    private Stmt constDeclaration() {
        Token name = consume(IDENTIFIER, "Expect constant name.");

        consume(EQUALS_SIGN, "Expect equals sign.");
        Expr initializer = expression();
        consume(SEMICOLON_SEPARATOR, "Expect ';' after variable declaration.");

        return new Stmt.Const(name, initializer);
    }

    /**
     * Matches an expression statement as specified in the grammar.cfg file.
     * 
     * @return An expression statement.
     */
    private Stmt expressionStatement() {
        Expr expr = expression();
        consume(SEMICOLON_SEPARATOR, "Expect ';' after expression.");
        return new Stmt.Expression(expr);
    }

    /**
     * Matches a block statement as specified in the grammar.cfg file.
     * 
     * @return A block statement.
     */
    private List<Stmt> block() {
        List<Stmt> statements = new ArrayList<>();

        while (!check(R_CURLY_BRACES) && !isAtEnd()) {
            statements.add(declaration());
        }

        consume(R_CURLY_BRACES, "Expect '}' after block.");
        return statements;
    }

    /**
     * Matches an assignment statement as specified in the grammar.cfg file.
     * 
     * @return An assignment statement.
     */
    private Expr assignment() {
        Expr expr = equality();

        if (match(EQUALS_SIGN)) {
            Token equals = previous();
            Expr value = assignment();

            if (expr instanceof Expr.Variable) {
                Token name = ((Expr.Variable) expr).name;
                return new Expr.Assign(name, value);
            }

            error(equals, "Invalid assignment target.");
        }

        return expr;
    }

    /**
     * Matches an expression as specified in the grammar.cfg file.
     * 
     * @return An expression.
     */
    private Expr expression() {
        return assignment();
    }

    /**
     * MAtches a declaration statement as specified in the grammar.cfg file.
     * 
     * @return A declaration expression
     */
    private Stmt declaration() {
        try {
            if (match(LET_KEYWORD))
                return varDeclaration();
            if (match(CONST_KEYWORD))
                return constDeclaration();

            return statement();
        } catch (ParseError error) {
            synchronize();
            return null;
        }
    }

    /**
     * Matches an equality expression as specified in the grammar.cfg file.
     * 
     * @return An equality expression.
     */
    private Expr equality() {
        Expr expr = comparison();

        while (match(LOGICAL_NOT_EQ, LOGICAL_EQ)) {
            Token operator = previous();
            Expr right = comparison();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    /**
     * Matches a comparison expression as specified in the grammar.cfg file.
     * 
     * @return A comparison expression.
     */
    private Expr comparison() {
        Expr term = logicOr();

        while (match(LESS_THAN, LESS_THAN_EQ, GREATER_THAN, GREATER_THAN_EQ)) {
            Token operator = previous();
            Expr right = logicOr();
            term = new Expr.Binary(term, operator, right);
        }

        return term;
    }

    /**
     * Matches a logical "OR" expression as specified in the grammar.cfg file.
     * 
     * @return A logical "OR" expression
     */
    private Expr logicOr() {
        Expr term = logicAnd();

        while (match(LOGICAL_OR)) {
            Token operator = previous();
            Expr right = logicAnd();
            term = new Expr.Binary(term, operator, right);
        }

        return term;
    }

    /**
     * Matches a logical "AND" expression as specified in the grammar.cfg file.
     * 
     * @return A logical "AND" expression
     */
    private Expr logicAnd() {
        Expr term = term();

        while (match(LOGICAL_AND)) {
            Token operator = previous();
            Expr right = term();
            term = new Expr.Binary(term, operator, right);
        }

        return term;
    }

    /**
     * Matches a term expression as specified in the grammar.cfg file.
     * 
     * @return A term expression.
     */
    private Expr term() {
        Expr expr = factor();

        while (match(MINUS, PLUS)) {
            Token operator = previous();
            Expr right = factor();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    /**
     * Matches a factor expression as specified in the grammar.cfg file.
     * 
     * @return A factor expression.
     */
    private Expr factor() {
        Expr expr = expo();

        while (match(DIV, MULT, MOD)) {
            Token operator = previous();
            Expr right = expo();
            expr = new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    /**
     * Matches a exponentiation expression as specified in the grammar.cfg file.
     * 
     * @return A exponentiation expression.
     */
    private Expr expo() {
        Expr expr = unary();

        while (match(EXPO)) {
            Token operator = previous();
            Expr right = unary();
            return new Expr.Binary(expr, operator, right);
        }

        return expr;
    }

    /**
     * Matches a unary expression as specified in the grammar.cfg file.
     * 
     * @return A unary expression.
     */
    private Expr unary() {
        if (match(LOGICAL_NOT, MINUS, PLUS)) {
            Token operator = previous();
            Expr right = unary();
            return new Expr.Unary(operator, right);
        } else {
            return primary();
        }
    }

    /**
     * Matches a primary (terminal) expression as specified in the grammar.cfg file.
     * These serve as a base-case for the recursive nature of the parser.
     * 
     * @return A primary (terminal) expression.
     */
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

        if (match(IDENTIFIER)) {
            return new Expr.Variable(previous());
        }

        if (match(L_PARENTHESIS)) {
            Expr expr = expression();
            consume(R_PARENTHESIS, "Expected ')' after expression.");
            return new Expr.Grouping(expr);
        }

        throw error(peek(), "Expect expression.");
    }
}