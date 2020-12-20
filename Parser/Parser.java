package org.hinton_lang.Parser;

import java.util.Arrays;
import java.io.IOException;
import java.nio.charset.Charset;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;

// Project-specific
import org.hinton_lang.Tokens.*;
import static org.hinton_lang.Tokens.TokenType.*;
import org.hinton_lang.Parser.AST.Expr;
import org.hinton_lang.Hinton;
import org.hinton_lang.Errors.ParseError;
import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Lexer.Lexer;
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
            statements.addAll(declaration());
        }

        return statements;
    }

    /**
     * Checks if the current token matches any of the provided tokens and consumes
     * it.
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

    /**
     * Matches a statement as specified in the grammar.cfg file.
     * 
     * @return A statement.
     */
    private Stmt statement() {
        if (match(L_CURLY_BRACES))
            return new Stmt.Block(block());
        if (match(IF_KEYWORD))
            return ifStatement();
        if (match(WHILE_KEYWORD))
            return whileStatement();
        if (match(FOR_KEYWORD))
            return forStatement();
        if (match(BREAK_KEYWORD))
            return breakStatement();
        if (match(CONTINUE_KEYWORD))
            return continueStatement();
        if (match(RETURN_KEYWORD))
            return returnStatement();
        if (match(IMPORT_KEYWORD))
            return importStatement();

        return expressionStatement();
    }

    /**
     * Matches a for-statement as specified in the grammar.cfg file.
     * 
     * @return A for-statement.
     */
    private Stmt forStatement() {
        consume(L_PARENTHESIS, "Expect '(' after 'for'.");

        // Gets the loop's initializer
        Stmt initializer;
        if (match(SEMICOLON_SEPARATOR)) {
            initializer = null;
        } else if (match(LET_KEYWORD)) {
            // In for-statements, we only accept single-variable
            // declarations. TODO: Fix this to throw an error when
            // the programmer tries to place a multi-variable
            // declaration in the for-loop's initializer.
            initializer = varDeclaration().get(0);
        } else {
            initializer = expressionStatement();
        }

        // Gets the condition for the loop
        Expr condition = null;
        if (!check(SEMICOLON_SEPARATOR)) {
            condition = expression();
        }
        consume(SEMICOLON_SEPARATOR, "Expect ';' after loop condition.");

        // Gets the increment statement for the loop
        Expr increment = null;
        if (!check(R_PARENTHESIS)) {
            increment = expression();
        }
        consume(R_PARENTHESIS, "Expect ')' after for clauses.");

        // Gets the loops body
        Stmt body = statement();

        // **** The following code converts the for-loop into a while-loop. ****

        // Adds the increment to the end of the loop's body
        if (increment != null) {
            body = new Stmt.Block(Arrays.asList(body, new Stmt.Expression(increment)));
        }

        // Creates a while loop with the condition and the body
        if (condition == null)
            condition = new Expr.Literal(true);
        body = new Stmt.While(condition, body);

        // Creates a block with the initializer, followed by the while loop
        if (initializer != null) {
            body = new Stmt.Block(Arrays.asList(initializer, body));
        }

        // Returns a while-loop version of the for-loop.
        // This is know an "syntactic sugar" in CS.
        return body;
    }

    /**
     * Matches a break statement as specified in the grammar.cfg file.
     * 
     * @return A break statement.
     */
    private Stmt breakStatement() {
        Token keyword = previous();
        match(SEMICOLON_SEPARATOR); // Optional semicolon
        return new Stmt.Break(keyword);
    }

    /**
     * Matches a continue statement as specified in the grammar.cfg file.
     * 
     * @return A continue statement.
     */
    private Stmt continueStatement() {
        Token keyword = previous();
        match(SEMICOLON_SEPARATOR); // Optional semicolon
        return new Stmt.Continue(keyword);
    }

    /**
     * Matches a return statement as specified in the grammar.cfg file.
     * 
     * @return A return statement.
     */
    private Stmt returnStatement() {
        Token keyword = previous();
        Expr value = expression();

        match(SEMICOLON_SEPARATOR); // Optional semicolon

        return new Stmt.Return(keyword, value);
    }

    /**
     * Matches an import statement as specified in the grammar.cfg file.
     * 
     * TODO: This is buggy. Finish the implementation to work as expected.
     * 
     * @return An import statement.
     */
    private Stmt importStatement() {
        consume(STRING_LITERAL, "Expected model path after import statement.");
        String path = (String) previous().literal;
        match(SEMICOLON_SEPARATOR); // Optional semicolon

        try {
            byte[] bytes = Files.readAllBytes(Paths.get(path));
            String sourceCode = new String(bytes, Charset.defaultCharset());

            Lexer lexer = new Lexer(sourceCode);
            List<Token> tokens = lexer.lexTokens();

            Parser parser = new Parser(tokens);
            List<Stmt> statements = parser.parse();

            return new Stmt.Import(statements);
        } catch (IOException e) {
            // TODO: Change to ParserError
            throw new RuntimeError("Cannot find module " + path);
        }
    }

    /**
     * Matches an if-statement as specified in the grammar.cfg file.
     * 
     * @return An if-statement.
     */
    private Stmt ifStatement() {
        consume(L_PARENTHESIS, "Expect '(' after 'if'.");
        Expr condition = expression();
        consume(R_PARENTHESIS, "Expect ')' after if condition.");

        Stmt thenBranch = statement();
        Stmt elseBranch = null;
        if (match(ELSE_KEYWORD)) {
            elseBranch = statement();
        }

        return new Stmt.If(condition, thenBranch, elseBranch);
    }

    /**
     * Matches a while-statement as specified in the grammar.cfg file.
     * 
     * @return An while-statement.
     */
    private Stmt whileStatement() {
        consume(L_PARENTHESIS, "Expect '(' after 'while'.");
        Expr condition = expression();
        consume(R_PARENTHESIS, "Expect ')' after condition.");
        Stmt body = statement();

        return new Stmt.While(condition, body);
    }

    /**
     * Matches a variable declaration as specified in the grammar.cfg file.
     * 
     * @return A variable declaration.
     */
    private ArrayList<Stmt> varDeclaration() {
        ArrayList<Token> declarations = new ArrayList<>();

        // Gets at least one variable name, or a list of
        // names separated by a comma
        declarations.add(consume(IDENTIFIER, "Expect variable name."));
        while (match(COMMA_SEPARATOR)) {
            declarations.add(consume(IDENTIFIER, "Expect variable name."));
        }

        // Since the .forEach loop bellow requires the
        // variables to be final, we use an array of size
        // one to represent the value of the variable.
        Expr initializer = null;
        if (match(EQUALS_SIGN)) {
            initializer = expression();
        }

        // Requires a semicolon at the end of the declaration
        // if the declaration was not a block
        if (previous().type != TokenType.R_CURLY_BRACES)
            consume(SEMICOLON_SEPARATOR, "Expect ';' after variable declaration.");

        // But if there is a semicolon after a curly brace, then we consume it
        if (previous().type == TokenType.R_CURLY_BRACES && check(SEMICOLON_SEPARATOR))
            advance();

        // Holds the declaration statements
        ArrayList<Stmt> statements = new ArrayList<>();

        // Assigns the value to the names.
        for (Token name : declarations) {
            statements.add(new Stmt.Var(name, initializer));
        }

        return statements;
    }

    /**
     * Matches a variable declaration as specified in the grammar.cfg file.
     * 
     * @return A variable declaration.
     */
    private ArrayList<Stmt> constDeclaration() {
        ArrayList<Token> declarations = new ArrayList<>();

        // Gets at least one constant name, or a list of
        // names separated by a comma
        declarations.add(consume(IDENTIFIER, "Expect constant name."));
        while (match(COMMA_SEPARATOR)) {
            declarations.add(consume(IDENTIFIER, "Expect constant name."));
        }

        // Gets the value
        match(EQUALS_SIGN);
        Expr initializer = expression();

        // Requires a semicolon at the end of the declaration
        // if the declaration was not a block
        if (previous().type != TokenType.R_CURLY_BRACES)
            consume(SEMICOLON_SEPARATOR, "Expect ';' after constant declaration.");

        // But if there is a semicolon after a curly brace, then we consume it
        if (check(SEMICOLON_SEPARATOR))
            advance();

        // Holds the declaration statements
        ArrayList<Stmt> statements = new ArrayList<>();

        // Assigns the value to the names.
        for (Token name : declarations) {
            statements.add(new Stmt.Var(name, initializer));
        }

        return statements;
    }

    /**
     * Matches an expression statement as specified in the grammar.cfg file.
     * 
     * @return An expression statement.
     */
    private Stmt expressionStatement() {
        Expr expr = expression();
        match(SEMICOLON_SEPARATOR); // Optional semicolon
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
            statements.addAll(declaration());
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
        Expr expr = logicOr();

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
    private ArrayList<Stmt> declaration() {
        try {
            ArrayList<Stmt> statements = new ArrayList<Stmt>();

            if (match(LET_KEYWORD)) {
                statements = varDeclaration();
            } else if (match(CONST_KEYWORD)) {
                statements = constDeclaration();
            } else if (match(FUNC_KEYWORD)) {
                statements.add(function("function"));
            } else if (match(CLASS_KEYWORD)) {
                statements.add(classDeclaration());
            } else {
                statements.add(statement());
            }

            return statements;
        } catch (ParseError error) {
            return null;
        }
    }

    /**
     * Matches a function statement as specified in the grammar.cfg file.
     * 
     * @param kind The type of function we wish to declare. Could be "function" for
     *             regular functions, or "method" for class methods.
     * @return A function declaration statement.
     */
    private Stmt.Function function(String kind) {
        Token name = consume(IDENTIFIER, "Expect " + kind + " name.");

        consume(L_PARENTHESIS, "Expect '(' after " + kind + " name.");
        List<Token> parameters = new ArrayList<>();
        if (!check(R_PARENTHESIS)) {
            do {
                if (parameters.size() >= 255) {
                    error(peek(), "Can't have more than 255 parameters.");
                }

                parameters.add(consume(IDENTIFIER, "Expect parameter name."));
            } while (match(COMMA_SEPARATOR));
        }
        consume(R_PARENTHESIS, "Expect ')' after parameters.");

        consume(L_CURLY_BRACES, "Expect '{' before " + kind + " body.");
        List<Stmt> body = block();
        return new Stmt.Function(name, parameters, body);
    }

    /**
     * Matches a class declaration as specified in the grammar.cfg file.
     * 
     * @return A class declaration.
     */
    private Stmt classDeclaration() {
        Token name = consume(IDENTIFIER, "Expect class name.");
        consume(L_CURLY_BRACES, "Expect '{' before class body.");

        List<Stmt.Function> methods = new ArrayList<>();
        while (!check(R_CURLY_BRACES) && !isAtEnd()) {
            consume(TokenType.FUNC_KEYWORD, "Expected function declaration.");
            methods.add(function("method"));
        }

        consume(R_CURLY_BRACES, "Expect '}' after class body.");

        return new Stmt.Class(name, methods);
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
        Expr term = term();

        while (match(LESS_THAN, LESS_THAN_EQ, GREATER_THAN, GREATER_THAN_EQ)) {
            Token operator = previous();
            Expr right = term();
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
            term = new Expr.Logical(term, operator, right);
        }

        return term;
    }

    /**
     * Matches a logical "AND" expression as specified in the grammar.cfg file.
     * 
     * @return A logical "AND" expression
     */
    private Expr logicAnd() {
        Expr term = equality();

        while (match(LOGICAL_AND)) {
            Token operator = previous();
            Expr right = equality();
            term = new Expr.Logical(term, operator, right);
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
        } else if (match(FUNC_KEYWORD)) {
            return lambda();
        } else if (match(NEW_KEYWORD)) {
            return instance();
        } else {
            Expr expr = primary();

            while (match(L_SQUARE_BRACKET, L_PARENTHESIS)) {
                // If there is an opening sqr bracket after the expression,
                // then we must have an array indexing expression.
                if (previous().type == L_SQUARE_BRACKET) {
                    expr = arrayIndexing(expr);
                }

                // If there is an opening parenthesis after the expression,
                // then we must have a function call expression.
                if (previous().type == L_PARENTHESIS) {
                    expr = call(expr);
                }
            }

            return expr;
        }
    }

    /**
     * Matches a function call expression as specified in the grammar.cfg file.
     * 
     * @return A function call expression.
     */
    private Expr call(Expr expr) {
        do {
            expr = finishCall(expr);
        } while (match(L_PARENTHESIS));

        return expr;
    }

    private Expr instance() {
        List<Expr> arguments = new ArrayList<>();
        Expr callee = primary();

        consume(L_PARENTHESIS, "Expected '(' before arguments.");

        if (!check(R_PARENTHESIS)) {
            do {
                // Hinton only supports 255 arguments for a class instance.
                if (arguments.size() >= 255) {
                    error(peek(), "Can't have more than 255 arguments.");
                }
                arguments.add(expression());
            } while (match(COMMA_SEPARATOR));
        }

        Token paren = consume(R_PARENTHESIS, "Expect ')' after arguments.");

        return new Expr.Instance(callee, paren, arguments);
    }

    /**
     * Matches a lambda expression as specified in the grammar.cfg file.
     * 
     * @return A lambda expression.
     */
    private Expr lambda() {
        consume(L_PARENTHESIS, "Expected '(' before parameters.");

        List<Token> parameters = new ArrayList<>();
        if (!check(R_PARENTHESIS)) {
            do {
                if (parameters.size() >= 255) {
                    error(peek(), "Can't have more than 255 parameters.");
                }

                parameters.add(consume(IDENTIFIER, "Expect parameter name."));
            } while (match(COMMA_SEPARATOR));
        }
        consume(R_PARENTHESIS, "Expected ')' for after parameters.");

        consume(L_CURLY_BRACES, "Expect '{' before function body.");
        List<Stmt> body = block();

        return new Expr.Lambda(parameters, body);
    }

    /**
     * Helper method to match a function call.
     * 
     * @param callee The called function.
     * @return A function call expression.
     */
    private Expr finishCall(Expr callee) {
        List<Expr> arguments = new ArrayList<>();

        if (!check(R_PARENTHESIS)) {
            do {
                // Hinton only supports 255 arguments for a function call.
                if (arguments.size() >= 255) {
                    error(peek(), "Can't have more than 255 arguments.");
                }
                arguments.add(expression());
            } while (match(COMMA_SEPARATOR));
        }

        Token paren = consume(R_PARENTHESIS, "Expect ')' after arguments.");

        return new Expr.Call(callee, paren, arguments);
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

        if (match(IDENTIFIER))
            return new Expr.Variable(previous());

        if (match(L_SQUARE_BRACKET))
            return constructArray();

        if (match(L_PARENTHESIS)) {
            Expr expr = expression();
            consume(R_PARENTHESIS, "Expected ')' after expression.");
            return new Expr.Grouping(expr);
        }

        throw error(peek(), "Expect expression.");
    }

    /**
     * Matches an array indexing expression as specified in the grammar.cfg file.
     * 
     * @param expr The identifier to be indexed.
     * @return An array indexing expression.
     */
    private Expr arrayIndexing(Expr expr) {
        do {
            expr = new Expr.Indexing(expr, expression());
            consume(R_SQUARE_BRACKET, "Expected ']' after array index.");
        } while (match(L_SQUARE_BRACKET));

        return expr;
    }

    /**
     * Constructs an array expression as specified in the grammar.cfg file.
     * 
     * @return An array expression.
     */
    private Expr constructArray() {
        ArrayList<Expr> expressions = new ArrayList<>();

        if (!match(R_SQUARE_BRACKET)) {
            expressions.add(expression());

            while (match(COMMA_SEPARATOR)) {
                expressions.add(expression());
            }

            consume(R_SQUARE_BRACKET, "Expected ']' after array declaration.");
        }

        return new Expr.Array(expressions);
    }
}
