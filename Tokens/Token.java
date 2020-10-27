package org.hinton_lang.Tokens;

import java.util.HashMap;


public class Token {
    public TokenType type;
    public int linePos;
    public int columnPos;
    public String text;
    public Object value;


    public Token(TokenType type, int linePos, int columnPos, String text, Object value) {
        this.type = type;
        this.linePos = linePos;
        this.columnPos = columnPos;
        this.text = text;
        this.value = value;
    }


    /**
     * String representation of the Token.
     * @return The string representation of the Token.
     */
    public String toString() {
        // The largest token's name length is 27
        String leftPad = " ".repeat(30 - type.name().length());
        return type.name() + ":" + leftPad + text;
    }


    /** Maps the keyword's names to their corresponding token types. */
    public static HashMap<String, TokenType> Keywords = new HashMap<>() {{
        // Control Flow
        // "while", "for", "in", "break", "continue", "repeat", "until"

        // // Logic flow and operators
        // "not", "equals", "is",
        put("if", TokenType.IF_KEYWORD);
        put("elif", TokenType.ELIF_KEYWORD);
        put("else", TokenType.ELSE_KEYWORD);
        put("and", TokenType.LOGICAL_AND);
        put("or", TokenType.LOGICAL_OR);
        put("not", TokenType.LOGICAL_NOT);
        put("equals", TokenType.LOGICAL_EQUALS);
        put("is", TokenType.LOGICAL_IS);
        put("mod", TokenType.ARITHMETIC_MODULUS);

        // // Module and OOP (no 'protected' keyword)
        // "import", "export", "new", "class", "readonly", "public",
        // "private", "self", "instanceof", "abstract", "implements",
        // "extends", "init", "static",

        // Static Types
        put("Int", TokenType.INTEGER_TYPE);
        put("NInt", TokenType.NINTEGER_TYPE);
        put("Real", TokenType.REAL_TYPE);
        put("NReal", TokenType.NREAL_TYPE);
        put("Char", TokenType.CHARACTER_TYPE);
        put("NChar", TokenType.CHARACTER_TYPE);
        put("String", TokenType.STRING_TYPE);
        put("NString", TokenType.NSTRING_TYPE);
        put("Bool", TokenType.BOOLEAN_TYPE);
        put("NBool", TokenType.NBOOLEAN_TYPE);
        put("Dict", TokenType.DICTIONARY_TYPE);
        put("NDict", TokenType.NDICTIONARY_TYPE);
        put("Set", TokenType.SET_TYPE);
        put("NSet", TokenType.NSET_TYPE);
        put("Function", TokenType.FUNCTION_TYPE);
        put("NFunction", TokenType.NFUNCTION_TYPE);
        put("void", TokenType.VOID_TYPE);
        put("any", TokenType.ANY_TYPE);
        put("None", TokenType.NONE_TYPE);

        // Keyword Literals
        put("true", TokenType.BOOLEAN_LITERAL_TRUE);
        put("false", TokenType.BOOLEAN_LITERAL_FALSE);
        put("none", TokenType.NONE_LITERAL);


        // Other
        // "async", "await", "as"
        put("pass", TokenType.PASS_STATEMENT);

        // Declarations
        // "return", "yield", "enum", "define",
        put("let", TokenType.LET_KEYWORD);
        put("const", TokenType.CONST_KEYWORD);
        put("func", TokenType.FUNC_KEYWORD);
    }};
}


;
