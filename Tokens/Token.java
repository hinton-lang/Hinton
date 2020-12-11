package org.hinton_lang.Tokens;

import java.util.HashMap;

public class Token {
    public TokenType type;
    public int linePos;
    public int columnPos;
    public String lexeme;
    public Object literal;

    public Token(TokenType type, int linePos, int columnPos, String text, Object value) {
        this.type = type;
        this.linePos = linePos;
        this.columnPos = columnPos;
        this.lexeme = text;
        this.literal = value;
    }

    /**
     * String representation of the Token.
     * 
     * @return The string representation of the Token.
     */
    public String toString() {
        // The largest token's name length is 27
        String leftPad = " ".repeat(30 - type.name().length());
        return type.name() + ":" + leftPad + lexeme;
    }

    /** Maps the keyword's names to their corresponding token types. */
    public static HashMap<String, TokenType> Keywords = new HashMap<>();

    /**
     * Static Constructor
     */
    static {
        // Control Flow
        // "loop" "in", "break", "continue",
        Keywords.put("for", TokenType.FOR_KEYWORD);
        Keywords.put("while", TokenType.WHILE_KEYWORD);

        // // Logic flow and operators
        // "not", "equals", "is",
        Keywords.put("if", TokenType.IF_KEYWORD);
        Keywords.put("elif", TokenType.ELIF_KEYWORD);
        Keywords.put("else", TokenType.ELSE_KEYWORD);
        Keywords.put("and", TokenType.LOGICAL_AND);
        Keywords.put("or", TokenType.LOGICAL_OR);
        Keywords.put("not", TokenType.LOGICAL_NOT);
        Keywords.put("equals", TokenType.LOGICAL_EQ);
        Keywords.put("is", TokenType.LOGICAL_IS);
        Keywords.put("mod", TokenType.MOD);

        // // Module and OOP (no 'protected' keyword)
        // "import", "export", "new", "class", "readonly", "public",
        // "private", "self", "instanceof", "abstract", "implements",
        // "extends", "init", "static",
        Keywords.put("class", TokenType.CLASS_KEYWORD);

        // Static Types
        Keywords.put("Int", TokenType.INTEGER_TYPE);
        Keywords.put("Real", TokenType.REAL_TYPE);
        Keywords.put("Char", TokenType.CHARACTER_TYPE);
        Keywords.put("String", TokenType.STRING_TYPE);
        Keywords.put("Bool", TokenType.BOOLEAN_TYPE);
        Keywords.put("Dict", TokenType.DICTIONARY_TYPE);
        Keywords.put("Set", TokenType.SET_TYPE);
        Keywords.put("Function", TokenType.FUNCTION_TYPE);
        Keywords.put("Void", TokenType.VOID_TYPE);
        Keywords.put("Any", TokenType.ANY_TYPE);
        Keywords.put("Null", TokenType.NULL_TYPE);

        // Keyword Literals
        Keywords.put("true", TokenType.BOOL_LITERAL_TRUE);
        Keywords.put("false", TokenType.BOOL_LITERAL_FALSE);
        Keywords.put("null", TokenType.NULL_LITERAL);

        // Other
        // "async", "await", "as"

        // Declarations
        // yield", "enum", "define",
        Keywords.put("let", TokenType.LET_KEYWORD);
        Keywords.put("const", TokenType.CONST_KEYWORD);
        Keywords.put("func", TokenType.FUNC_KEYWORD);
        Keywords.put("return", TokenType.RETURN_KEYWORD);
    }
}

;
