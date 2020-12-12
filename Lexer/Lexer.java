package org.hinton_lang.Lexer;

import java.util.ArrayList;
import java.util.Scanner;

// Project-specific
import org.hinton_lang.Helper.Regex;
import org.hinton_lang.Tokens.*;
import static org.hinton_lang.Tokens.TokenType.*;
import org.hinton_lang.Errors.IllegalTokenError;

/**
 * Lexical Analyzer that takes an input file and converts the contents of the
 * file into a list of tokens based on the rules defined by the Tokenizer.
 */
public class Lexer {
    private final String sourceCode;
    public int currLineNum = 1;
    public int currCharPos = 0;
    private int currLineLength = 0;
    private String currLineText = "";
    public ArrayList<Token> TokensList = new ArrayList<>();
    private boolean isInsideBlockComment = false;

    /**
     * Lexical Analyzer that takes an input file and converts the contents of the
     * file into a list of tokens based on the rules defined by the Tokenizer.
     * 
     * @param source The raw source text.
     */
    public Lexer(String source) {
        this.sourceCode = source;
    }

    /**
     * Generates a list of tokens from the input file.
     * 
     * @return A ArrayList<Token> that contains the generated tokens.
     */
    public ArrayList<Token> lexTokens() {
        Scanner reader = new Scanner(sourceCode);

        for (currLineNum = 1; reader.hasNextLine(); currLineNum++) {
            this.analyzeLine(reader);
        }

        return TokensList;
    }

    private char prevChar() {
        return currLineText.toCharArray()[this.currCharPos - 1];
    }

    protected char nextChar() {
        return currLineText.toCharArray()[this.currCharPos + 1];
    }

    protected boolean hasNextChar() {
        return currCharPos < currLineText.length();
    }

    private boolean isIllegalIdentifierChar(char currentChar) {
        return Regex.Match(currentChar, "(@|%|!|&|\\*|\\(|\\)|-|\\+|=|\\{|\\}|\\[|\\]|\\||;|:|<|>|,|\\.|\\/|\\?|`|~)");
    }

    /**
     * Analyzes each line of the input file
     * 
     * @param reader The File Scanner's reader.
     */
    private void analyzeLine(Scanner reader) {
        String lineData = reader.nextLine();
        // codeLines.add(lineData); // Add the line to the code lines.
        this.currLineText = lineData;
        boolean isLastLine = !reader.hasNextLine();
        boolean skippedLine = false;

        // **** EMPTY LINES ****
        if (lineData.equals("") && !isLastLine)
            return;

        // **** LINE COMMENTS ****
        if (lineData.trim().startsWith("//"))
            skippedLine = true;
        // **** SINGLE-LINE BLOCK COMMENTS
        if (lineData.trim().startsWith("/*") && lineData.trim().endsWith("*/"))
            skippedLine = true;

        // Analyze the line if it is not a comment
        if (!skippedLine) {
            // Finds tokens in the current line
            currLineLength = lineData.length();
            this.Tokenize(lineData);
        }

        // End of file
        if (isLastLine) {
            Token currentToken = new Token(END_OF_FILE, currLineNum, currCharPos, "EOF", null);
            this.TokensList.add(currentToken);
        }
    }

    /**
     * Tokenize the sequences of characters in each line.
     * 
     * @param lineData The line to tokenize.
     * @throws IllegalTokenError If the character is not recognized by the
     *                           tokenizer.
     */
    private void Tokenize(String lineData) throws IllegalTokenError {
        char[] line = lineData.toCharArray();

        for (this.currCharPos = 0; this.currCharPos < line.length; this.currCharPos++) {
            char currentChar = line[this.currCharPos];

            // **** Whitespace
            if (Character.isWhitespace(currentChar))
                continue;
            // **** Inline Comments
            if (currentChar == '/' && this.nextChar() == '/')
                break;
            // **** Multiline Block Comments
            if (currentChar == '/' && nextChar() == '*') {
                this.currCharPos++; // skip the first comment star
                this.isInsideBlockComment = true;
            }
            if (isInsideBlockComment) {
                if (currentChar == '*' && nextChar() == '/') {
                    this.isInsideBlockComment = false;
                }

                this.currCharPos++;
                continue;
            }

            // **** Arithmetic Operators, i.e., + - * / ** %
            final String arithmeticOperator = "(\\+|-|\\*|%)";
            if (Regex.Match(currentChar, arithmeticOperator) || (currentChar == '/' && this.nextChar() != '/')) {
                this.AddArithmeticOperatorToken(currentChar);
                continue;
            }

            // **** Logical Operators, i.e., > = < ! & | ~ ^
            final String logicalOperator = ">|=|<|!|&|~|\\^|\\|";
            if (Regex.Match(currentChar, logicalOperator)) {
                LogicalOrBitwiseOperator.add(this, currentChar);
                continue;
            }

            // **** Separators & Paired-Delimiters, i.e., ( ) { } ; , [ ] : .
            final String separator = "\\(|\\)|\\{|\\}|;|,|\\[|\\]|:";
            if (Regex.Match(currentChar, separator) || (currentChar == '.' && !Character.isDigit(nextChar()))) {
                this.AddSeparatorAndPunctuationToken(currentChar);
                continue;
            }

            // **** Numeric Literals
            if (Character.isDigit(currentChar) || (currentChar == '.' && Character.isDigit(nextChar()))) {
                this.AddNumericLiteralToken(line);
                continue;
            }

            // **** String Literals
            if (currentChar == '"') {
                this.AddStringLiteralToken(line);
                continue;
            }

            // **** Keywords & Identifiers
            if (Character.isAlphabetic(currentChar) || currentChar == '_' || currentChar == '$') {
                this.AddKeywordsOrIdentifierTokens(line);
                continue;
            }

            // **** Bad Character
            Token currentToken = new Token(BAD_CHARACTER, currLineNum, currCharPos, Character.toString(currentChar),
                    null);
            throw new IllegalTokenError(currentToken);
        }
    }

    /**
     * Adds an arithmetic operator to the tokens list.
     * 
     * @param operator The current character
     */
    private void AddArithmeticOperatorToken(char operator) {
        TokenType tokenType;

        if (operator == '+') {
            tokenType = PLUS;
        } else if (operator == '-') {
            tokenType = MINUS;
        } else if (operator == '*' && this.nextChar() != '*') {
            tokenType = MULT;
        } else if (operator == '*' && this.nextChar() == '*') {
            tokenType = EXPO;
        } else if (operator == '/') {
            tokenType = DIV;
        } else {
            tokenType = MOD;
        }

        String tokenChar = (operator == '*' && this.nextChar() == '*') ? "**" : Character.toString(operator);
        Token currentToken = new Token(tokenType, currLineNum, this.currCharPos, tokenChar, null);
        if (operator == '*' && this.nextChar() == '*')
            this.currCharPos++;
        this.TokensList.add(currentToken);
    }

    /**
     * Adds a separator token or a paired-delimiter to the tokens list
     * 
     * @param op The current character.
     */
    private void AddSeparatorAndPunctuationToken(char op) {
        TokenType tokenType;

        // ( ) { } ; , [ ] : .
        if (op == '(') {
            tokenType = L_PARENTHESIS;
        } else if (op == ')') {
            tokenType = R_PARENTHESIS;
        } else if (op == '{') {
            tokenType = L_CURLY_BRACES;
        } else if (op == '}') {
            tokenType = R_CURLY_BRACES;
        } else if (op == ';') {
            tokenType = SEMICOLON_SEPARATOR;
        } else if (op == ',') {
            tokenType = COMMA_SEPARATOR;
        } else if (op == '[') {
            tokenType = L_SQUARE_BRAKET;
        } else if (op == ']') {
            tokenType = R_SQUARE_BRAKET;
        } else if (op == ':') {
            tokenType = COLON_SEPARATOR;
        } else if (op == '.' && hasNextChar() && nextChar() == '.') {
            tokenType = RANGE_OPERATOR;
        } else {
            tokenType = DOT_SEPARATOR;
        }

        String opText = (tokenType == RANGE_OPERATOR) ? ".." : Character.toString(op);
        Token currentToken = new Token(tokenType, this.currLineNum, this.currCharPos, opText, null);
        this.TokensList.add(currentToken);

        // Accounts for the fact that the range operator takes two characters
        if (tokenType == RANGE_OPERATOR)
            currCharPos++;
    }

    /**
     * Adds a numeric literal to the tokens list
     * 
     * @param charSequence The current character
     */
    private void AddNumericLiteralToken(char[] charSequence) {
        final int start = this.currCharPos;
        char currentChar;
        StringBuilder numString = new StringBuilder();
        boolean foundFloat = false;

        while (currCharPos < currLineLength) {
            currentChar = charSequence[this.currCharPos];

            if (Character.isWhitespace(currentChar))
                break;
            // Support for range operator.
            if (hasNextChar() && currentChar == '.' && nextChar() == '.')
                break;
            // Support for real numbers and numbers with underscores.
            if (!Regex.Match(currentChar, "[0-9]|_|\\.") || (currentChar == '.' && foundFloat))
                break;
            if (currentChar == '.')
                foundFloat = true;

            numString.append(currentChar);
            currCharPos++;
        }

        // Adjusts the currentCharPos for the Tokenizer's for-loop.
        currCharPos--;

        // Removes the underscores.
        StringBuilder noUnderscores = new StringBuilder();
        for (char digit : numString.toString().toCharArray()) {
            if (digit == '_')
                continue;
            noUnderscores.append(digit);
        }

        // Converts the string to a number. If it has a floating point, it
        // will be kept as a double, otherwise, it will be casted to an int.
        double doubleVal = Double.parseDouble(noUnderscores.toString());

        // The number's representation in the language
        TokenType numType = (!foundFloat) ? INTEGER_LITERAL : REAL_LITERAL;

        Token currentToken;
        if (!foundFloat) {
            currentToken = new Token(numType, this.currLineNum, start, noUnderscores.toString(), (int) doubleVal);
        } else {
            currentToken = new Token(numType, this.currLineNum, start, noUnderscores.toString(), doubleVal);
        }
        this.TokensList.add(currentToken);
    }

    /**
     * Adds a string literal to the tokens list.
     * 
     * @param charSequence The current line
     */
    private void AddStringLiteralToken(char[] charSequence) {
        final int start = this.currCharPos;
        char currentChar;
        StringBuilder stringLiteral = new StringBuilder();

        while (currCharPos < currLineLength) {
            currentChar = charSequence[this.currCharPos];
            stringLiteral.append(currentChar);

            // Breaks the loop when it finds the closing quotes, and the found quotes are
            // not escaped.
            if ((currentChar == '"' && currCharPos != start) && this.prevChar() != '\\')
                break;

            currCharPos++;
        }

        Token currentToken = new Token(STRING_LITERAL, this.currLineNum, start, stringLiteral.toString(),
                stringLiteral);
        this.TokensList.add(currentToken);
    }

    /**
     * Finds keywords or identifiers in the code. If a word outside quotes is not a
     * keyword, then it must be an identifier created by the programmer.
     * 
     * @param charSequence The current line
     */
    private void AddKeywordsOrIdentifierTokens(char[] charSequence) {
        final int start = this.currCharPos;
        char currentChar;
        StringBuilder keyword = new StringBuilder();

        while (currCharPos < currLineLength) {
            currentChar = charSequence[this.currCharPos];

            if (Character.isWhitespace(currentChar) || isIllegalIdentifierChar(currentChar))
                break;
            keyword.append(currentChar);

            currCharPos++;
        }

        // Adjusts the currentCharPos for the Tokenizer's for-loop.
        currCharPos--;

        Token currentToken;
        if (Token.Keywords.containsKey(keyword.toString())) {
            TokenType kwType = Token.Keywords.get(keyword.toString());
            currentToken = new Token(kwType, currLineNum, start, keyword.toString(), null);
        } else {
            currentToken = new Token(CUSTOM_IDENTIFIER, currLineNum, start, keyword.toString(), null);
        }
        TokensList.add(currentToken);
    }
}
