package org.hinton_lang.Parser;

import java.util.ArrayList;
import java.util.Arrays;

// Project Packages
import org.hinton_lang.AbstractSyntaxTree.*;
import org.hinton_lang.Helper.Regex;
import org.hinton_lang.Lexer.Lexer;
import org.hinton_lang.Symbols.AssignmentType;
import org.hinton_lang.Symbols.BuiltInTypes;
import org.hinton_lang.Tokens.Token;
import org.hinton_lang.Tokens.TokenType;
import org.hinton_lang.Errors.UnexpectedTokenError;


public class Parser {
    public ArrayList<Token> Tokens;
    private int currentTokenPos = 0;

    public Parser(String filePath) {
        // Generates the tokens from the input file.
        this.Tokens = new Lexer(filePath).lex();
        
        // for (Token t: Tokens) System.out.println(t);
    }


    /**
     * Parses the tokens and generates the Abstract Syntax Trees to be consumed
     * by the interpreter.
     */
    public Program parse() throws UnexpectedTokenError {
        ArrayList<AST> program_statements = this.statement_list();

        if (this.currentToken().type != TokenType.END_OF_FILE) {
            throw new UnexpectedTokenError(this.currentToken());
        } else {
            return new Program(program_statements);
        }
    }


    /** Gets the current token (without incrementing currentTokenPos) */
    private Token currentToken() {
        return this.Tokens.get(currentTokenPos);
    }


    /** Gets the next token (without incrementing currentTokenPos) */
    private Token next_token() {
        return this.Tokens.get(this.currentTokenPos + 1);
    }


    /** Checks if there is still a token ahead */
    private Boolean hasNextToken() {
        return this.currentTokenPos < this.Tokens.size();
    }


    /**
     * Deletes the current token if it matches the passed type, and moves the imaginary
     * pointer forward
     * @param type The expected type of the token to delete.
     * @throws Error if the type of the current token does not match the expected type.
     */
    private void delete(TokenType type) throws UnexpectedTokenError {
        // console.log(`Current Token: ${Object.values(TokenType)[this.current_token.type]},    Expecting: ${Object.values(TokenType)[type]}`)
        if (this.currentToken().type == type) {
            this.currentTokenPos++;
        } else {
            throw new UnexpectedTokenError(this.currentToken());
        }
    }


    /**
     * Generates the Abstract Syntax Trees for all statements in the input program.
     * Grammar Rule -> statement_list: statement | statement SEMI statement_list
     */
    private ArrayList<AST> statement_list() {
        ArrayList<AST> results = new ArrayList<>();

        Token token = this.currentToken();

        if (token.type == TokenType.OPEN_CURLY_BRACES) {
            results.add(this.compound_statement());
        } else {
            results.addAll(this.statement());

            token = this.currentToken();

            while (token.type == TokenType.SEMICOLON_SEPARATOR) {
                this.delete(TokenType.SEMICOLON_SEPARATOR);
                results.addAll(this.statement_list());
                token = this.currentToken();
            }
        }

        return results;
    }


    /**
     * Generates Abstract Syntax Trees for different statements.
     * Rule -> statement: compound_statement | declarations | statement_list | empty
     */
    private ArrayList<AST> statement() {
        Token token = this.currentToken();
        ArrayList<AST> nodes = new ArrayList<>();

        TokenType[] declarationTypes = {
            TokenType.LET_KEYWORD, // declaring a variable
            TokenType.CONST_KEYWORD, // declaring a constant
            TokenType.FUNC_KEYWORD, // declaring a function
        };

        if (Arrays.asList(declarationTypes).contains(token.type)) {
            nodes.addAll(this.declarations());
        } else if (token.type == TokenType.CUSTOM_IDENTIFIER) {
            nodes.addAll(this.reDeclarations());
        } 
        // else if (token.type == TokenType.OPEN_CURLY_BRACES) {
        //     nodes.add(this.compound_statement());
        // }
        else {
            // Standalone expressions allow having expressions without declaring a variable.
            // To allow standalone expressions in the code, uncomment the following line,
            // and comment the line bellow it.
            // nodes.add(this.expr());
            nodes.add(this.empty());
        }

        return nodes;
    }


    /**
     * Generates an Abstract Syntax Tree for a compound statement block.
     * Rule -> compound_statement: LCURLY statement_list RCURLY
     */
    private Compound compound_statement() {
        this.delete(TokenType.OPEN_CURLY_BRACES);
        ArrayList<AST> nodes = this.statement_list();
        this.delete(TokenType.CLOSE_CURLY_BRACES);

        return new Compound(nodes);
    }


    /**
     * Rule -> declarations: (var_declarations SEMI)+ | (const_declarations SEMI)+
     */
    private ArrayList<AST> declarations() {
        ArrayList<AST> declarations = new ArrayList<>();

        switch (this.currentToken().type) {
            // For variable and Constant declarations
            case LET_KEYWORD:
                declarations.addAll(this.var_declarations());
                break;
            case CONST_KEYWORD:
                declarations.addAll(this.const_declarations());
                break;
            case FUNC_KEYWORD:
                declarations.add(this.func_declaration());
                break;
            default: break;
        }

        return declarations;
    }


    /**
     * Rule -> reDeclaration: identifier (COMMA identifier)* ASSIGN expr
     */
    private ArrayList<AST> reDeclarations() {
        ArrayList<Identifier> identifiers = new ArrayList<>();
        ArrayList<AST> nodes = new ArrayList<>();

        Token token = this.currentToken();
        while ((token.type == TokenType.CUSTOM_IDENTIFIER || token.type == TokenType.COMMA_SEPARATOR)) {
            if (token.type == TokenType.COMMA_SEPARATOR) {
                this.delete(TokenType.COMMA_SEPARATOR);
            } else {
                identifiers.add(this.identifier());
            }

            // Updates the current token
            token = this.currentToken();
        }

        this.delete(TokenType.EQUALS_SIGN);
        AST right = this.expr();

        // Assigns the right-hand-side of the variable declaration
        // to each of the identifiers in the left-hand-side.
        for (Identifier id: identifiers) {
            VarDeclaration left = new VarDeclaration(id, new TypeDef(BuiltInTypes.ANY));
            ReAssignment node = new ReAssignment(AssignmentType.VARIABLE, left, right);
            nodes.add(node);
        }

        return nodes;
    }


    /**
     * Rule -> variable_declaration: LET identifier (COMMA identifier)* (COLON type_spec) ASSIGN expr
     */
    private ArrayList<AST> var_declarations() {
        this.delete(TokenType.LET_KEYWORD);

        ArrayList<Identifier> identifiers = new ArrayList<>();
        ArrayList<AST> nodes = new ArrayList<>();

        Token token = this.currentToken();
        while ((token.type == TokenType.CUSTOM_IDENTIFIER || token.type == TokenType.COMMA_SEPARATOR)) {
            if (token.type == TokenType.COMMA_SEPARATOR) {
                this.delete(TokenType.COMMA_SEPARATOR);
            } else {
                identifiers.add(this.identifier());
            }

            // Updates the current token
            token = this.currentToken();
        }

        // The type definition for this variable
        TypeDef exprType = (this.currentToken().type == TokenType.COLON_SEPARATOR)
                                ? this.type_spec()
                                : new TypeDef(BuiltInTypes.ANY);

        AST right;
        if (this.currentToken().type == TokenType.EQUALS_SIGN) {
            this.delete(TokenType.EQUALS_SIGN);
            right = this.expr();
        } else {
            right = this.empty();
        }

        // Assigns the right-hand-side of the variable declaration
        // to each of the identifiers in the left-hand-side.
        for (Identifier id: identifiers) {
            VarDeclaration left = new VarDeclaration(id, exprType);
            Assignment node = new Assignment(AssignmentType.VARIABLE, left, right);
            nodes.add(node);
        }

        return nodes;
    }


    /**
     * Rule -> const_declaration: CONST identifier (COMMA identifier)* (COLON type_spec) ASSIGN expr
     */
    private ArrayList<AST> const_declarations() {
        this.delete(TokenType.CONST_KEYWORD);

        ArrayList<Identifier> identifiers = new ArrayList<>();
        ArrayList<AST> nodes = new ArrayList<>();

        Token token = this.currentToken();
        while ((token.type == TokenType.CUSTOM_IDENTIFIER || token.type == TokenType.COMMA_SEPARATOR)) {
            if (token.type == TokenType.COMMA_SEPARATOR) {
                this.delete(TokenType.COMMA_SEPARATOR);
            } else {
                identifiers.add(this.identifier());
            }

            // Updates the current token
            token = this.currentToken();
        }


        // The type definition for this constant
        // Constants must have a type definition upon declaration
        TypeDef exprType = this.type_spec();

        // Constant must have a value  upon declaration
        this.delete(TokenType.EQUALS_SIGN);
        AST right = this.expr();

        // Assigns the right-hand-side of the variable declaration
        // to each of the identifiers in the left-hand-side.
        for (Identifier id: identifiers) {
            ConstDeclaration left = new ConstDeclaration(id, exprType);
            Assignment node = new Assignment(AssignmentType.CONSTANT, left, right);
            nodes.add(node);
        }

        return nodes;
    }


    /**
     * Rule -> FUNC identifier LPAREN RPAREN COLON type_spec compound_statement
     */
    private AST func_declaration() {
        this.delete(TokenType.FUNC_KEYWORD);

        // The function's name
        Identifier funcID = this.identifier();

        this.delete(TokenType.OPEN_PARENTHESIS);
        this.delete(TokenType.CLOSE_PARENTHESIS);

        // The function's return type
        TypeDef funcReturnType = new TypeDef(BuiltInTypes.ANY);
        if (currentToken().type == TokenType.COLON_SEPARATOR) funcReturnType = this.type_spec();

        // Compose the function declaration and assignment token
        FuncDeclaration funcDec = new FuncDeclaration(funcID, funcReturnType);
        return new Assignment(AssignmentType.FUNCTION, funcDec, this.compound_statement());
    }


    /**
     * Rule -> type_spec: INTEGER | REAL
     */
    private TypeDef type_spec() {
        this.delete(TokenType.COLON_SEPARATOR);
        if (this.currentToken().type == TokenType.INTEGER_TYPE) {
            this.delete(TokenType.INTEGER_TYPE);
            return new TypeDef(BuiltInTypes.INT);
        } if (this.currentToken().type == TokenType.REAL_TYPE) {
            this.delete(TokenType.REAL_TYPE);
            return new TypeDef(BuiltInTypes.REAL);
        } if (this.currentToken().type == TokenType.STRING_TYPE) {
            this.delete(TokenType.STRING_TYPE);
            return new TypeDef(BuiltInTypes.STRING);
        } if (this.currentToken().type == TokenType.CHARACTER_TYPE) {
            this.delete(TokenType.CHARACTER_TYPE);
            return new TypeDef(BuiltInTypes.CHAR);
        } if (this.currentToken().type == TokenType.BOOLEAN_TYPE) {
            this.delete(TokenType.BOOLEAN_TYPE);
            return new TypeDef(BuiltInTypes.BOOLEAN);
        } if (this.currentToken().type == TokenType.VOID_TYPE) {
            this.delete(TokenType.VOID_TYPE);
            return new TypeDef(BuiltInTypes.VOID);
        } if (this.currentToken().type == TokenType.NULL_TYPE) {
            this.delete(TokenType.NULL_TYPE);
            return new TypeDef(BuiltInTypes.NULL);
        } else {
            this.delete(TokenType.ANY_TYPE);
            return new TypeDef(BuiltInTypes.ANY);
        }
    }


    /**
     * Rule -> expr: term ((PLUS | MINUS) term)*
     */
    private AST expr() {
        // Gets the first integer, and moves the imaginary pointer
        AST node = this.term();
        Token token = this.currentToken();

        while (token.type == TokenType.ARITHMETIC_PLUS || token.type == TokenType.ARITHMETIC_MINUS) {
            if (token.type == TokenType.ARITHMETIC_PLUS) {
                this.delete(TokenType.ARITHMETIC_PLUS);
            } else {
                this.delete(TokenType.ARITHMETIC_MINUS);
            }

            node = new BinaryOperator(node, token, this.term());
            token = this.currentToken();
        }

        return node;
    }


    /**
     * Rule-> term: factor ((MUL | DIV | MOD) factor)*
     */
    private AST term() {
        AST node = this.factor();
        Token token = this.currentToken();

        while (Regex.Match(token.text, "\\*{1}|\\/|%|mod")) {
            if (token.type == TokenType.ARITHMETIC_MULT) {
                this.delete(TokenType.ARITHMETIC_MULT);
            } else if (token.type == TokenType.ARITHMETIC_DIVISION) {
                this.delete(TokenType.ARITHMETIC_DIVISION);
            } else {
                this.delete(TokenType.ARITHMETIC_MODULUS);
            }

            node = new BinaryOperator(node, token, this.factor());
            token = this.currentToken();
        }

        return node;
    }


    /**
     * Rule -> factor: atom (POW factor)*
     */
    private AST factor() {
        AST node = this.atom();
        Token token = this.currentToken();

        while (token.type == TokenType.ARITHMETIC_EXPONENT) {
            this.delete(TokenType.ARITHMETIC_EXPONENT);
            node = new BinaryOperator(node, token, this.atom());
            token = this.currentToken();
        }

        return node;
    }


    /**
     * Rule -> atom: PLUS factor | MINUS factor | INTEGER | LPAREN expr RPAREN | identifier
     */
    private AST atom() {
        Token token = this.currentToken();
        AST node;

        switch (token.type) {
            case INTEGER_LITERAL:
                this.delete(TokenType.INTEGER_LITERAL);
                node = new IntLiteral(token);
                break;
            case REAL_LITERAL:
                this.delete(TokenType.REAL_LITERAL);
                node = new RealLiteral(token);
                break;
            case STRING_LITERAL:
                this.delete(TokenType.STRING_LITERAL);
                node = new StringLiteral(token);
                break;
            case BOOLEAN_LITERAL_TRUE:
                this.delete(TokenType.BOOLEAN_LITERAL_TRUE);
                node = new BoolLiteral(token);
                break;
            case BOOLEAN_LITERAL_FALSE:
                this.delete(TokenType.BOOLEAN_LITERAL_FALSE);
                node = new BoolLiteral(token);
                break;
            case ARITHMETIC_PLUS:
                this.delete(TokenType.ARITHMETIC_PLUS);
                node = new UnaryOperator(token, this.factor());
                break;
            case ARITHMETIC_MINUS:
                this.delete(TokenType.ARITHMETIC_MINUS);
                node = new UnaryOperator(token, this.factor());
                break;
            case OPEN_PARENTHESIS:
                this.delete(TokenType.OPEN_PARENTHESIS);
                AST n = this.expr();
                this.delete(TokenType.CLOSE_PARENTHESIS);
                node = n;
                break;
            case NULL_LITERAL:
                this.delete(TokenType.NULL_LITERAL);
                node = this.empty();
                break;
            default:
                node = this.identifier();
        }

        return node;
    }


    /**
     * Rule -> identifier: ID
     */
    private Identifier identifier() {
        Identifier node = new Identifier(this.currentToken());
        this.delete(TokenType.CUSTOM_IDENTIFIER);
        return node;
    }


    /**
     * Rule -> empty:
     */
    private AST empty() { return new NoOp(); }
}
