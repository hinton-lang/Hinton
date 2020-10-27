package org.hinton_lang.Lexer;

import org.hinton_lang.Helper.Regex;
import org.hinton_lang.Tokens.Token;
import org.hinton_lang.Tokens.TokenType;
import org.hinton_lang.Errors.IllegalTokenError;

import java.io.File;
import java.io.FileNotFoundException;
import java.util.ArrayList;
import java.util.Scanner;
// import java.util.Vector;


/**
 * Lexical Analyzer that takes an input file and converts the contents
 * of the file into a list of tokens based on the rules defined by the Tokenizer.
 */
public class Lexer {
    private final String filePath;
    //public Vector<String> codeLines = new Vector<>();
    public int currentLineNum = 1;
    public int currentCharPos = 0;
    private int currentLineLength = 0;
    private String currentLineText = "";
    public ArrayList<Token> TokensList = new ArrayList<>();


    /**
     * Lexical Analyzer that takes an input file and converts the contents
     * of the file into a list of tokens based on the rules defined by the Tokenizer.
     * @param filePath The path to the file that will be tokenized.
     */
    public Lexer(String filePath) { this.filePath = filePath; }


    /**
     * Generates a list of tokens from the input file.
     * @return A ArrayList<Token> that contains the generated tokens.
     */
    public ArrayList<Token> lex() {
        try {
            File testFile = new File(filePath);
            Scanner reader = new Scanner(testFile);
            for (currentLineNum = 1; reader.hasNextLine(); currentLineNum++) this.analyzeLine(reader);
            reader.close();
        } catch (FileNotFoundException e) {
            System.out.println("File Not Found!!");
        }

        return TokensList;
    }


    private char prevChar() {
        return currentLineText.toCharArray()[this.currentCharPos - 1];
    }

    protected char nextChar() {
        return currentLineText.toCharArray()[this.currentCharPos + 1];
    }

    protected boolean hasNextChar() {
        return currentCharPos < currentLineText.length();
    }

    private boolean isIllegalIdentifierChar(char currentChar) {
        return Regex.Match(currentChar, "(@|%|!|&|\\*|\\(|\\)|-|\\+|=|\\{|\\}|\\[|\\]|\\||;|:|<|>|,|\\.|\\/|\\?|`|~)");
    }


    /**
     * Analyzes each line of the input file
     * @param reader The File Scanner's reader.
     */
    private void analyzeLine(Scanner reader) {
        String lineData = reader.nextLine();
        //codeLines.add(lineData); // Add the line to the code lines.
        this.currentLineText = lineData;
        boolean isLastLine = !reader.hasNextLine();
        boolean skippedLine = false;

        // **** EMPTY LINES ****
        if (lineData.equals("") && !isLastLine) return;

        // **** LINE COMMENTS ****
        if (lineData.trim().startsWith("//")) skippedLine = true;
        // **** SINGLE-LINE BLOCK COMMENTS
        if (lineData.trim().startsWith("/*") && lineData.trim().endsWith("*/")) skippedLine = true;

        // Analyze the line if it is not a comment
        if (!skippedLine) {
            // Finds tokens in the current line
            currentLineLength = lineData.length();
            this.Tokenize(lineData);
        }

        // End of file
        if (isLastLine) {
            Token currentToken = new Token(TokenType.END_OF_FILE, currentLineNum, currentCharPos, "EOF", null);
            this.TokensList.add(currentToken);
        }
    }


    /**
     * Tokenize the sequences of characters in each line.
     * @param lineData The line to tokenize.
     * @throws IllegalTokenError If the character is not recognized by the tokenizer.
     */
    private void Tokenize(String lineData) throws IllegalTokenError {
        char[] line = lineData.toCharArray();

        for (this.currentCharPos = 0; this.currentCharPos < line.length; this.currentCharPos++) {
            char currentChar = line[this.currentCharPos];

            // **** Whitespace
            if (Character.isWhitespace(currentChar)) continue;
            // **** Inline Comments
            if (currentChar == '/' && this.nextChar() == '/') break;


            // **** Arithmetic Operators, i.e.,   + - * / ** %
            final String arithmeticOperator = "(\\+|-|\\*|%)";
            if (Regex.Match(currentChar, arithmeticOperator) || (currentChar == '/' && this.nextChar() != '/')) {
                this.AddArithmeticOperatorToken(currentChar);
                continue;
            }


            // **** Logical Operators, i.e.,   > = < ! & | ~ ^
            final String logicalOperator = ">|=|<|!|&|~|\\^|\\|";
            if (Regex.Match(currentChar, logicalOperator)) {
                LogicalOrBitwiseOperator.add(this, currentChar);
                continue;
            }


            // **** Separators & Paired-Delimiters, i.e.,   ( ) { } ; , [ ] : .
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
            Token currentToken = new Token(TokenType.BAD_CHARACTER, currentLineNum, currentCharPos,
                    Character.toString(currentChar), null);
            throw new IllegalTokenError(currentToken);
        }
    }


    /**
     * Adds an arithmetic operator to the tokens list.
     * @param operator The current character
     */
    private void AddArithmeticOperatorToken(char operator) {
        TokenType tokenType;

        if (operator == '+') { tokenType = TokenType.ARITHMETIC_PLUS; }
        else if (operator == '-') { tokenType = TokenType.ARITHMETIC_MINUS; }
        else if (operator == '*' && this.nextChar() != '*') { tokenType = TokenType.ARITHMETIC_MULT; }
        else if (operator == '*' && this.nextChar() == '*') { tokenType = TokenType.ARITHMETIC_EXPONENT; }
        else if (operator == '/') { tokenType = TokenType.ARITHMETIC_DIVISION; }
        else { tokenType = TokenType.ARITHMETIC_MODULUS; }

        String tokenChar = (operator == '*' && this.nextChar() == '*') ? "**" : Character.toString(operator);
        Token currentToken = new Token(tokenType, currentLineNum, this.currentCharPos, tokenChar, null);
        if (operator == '*' && this.nextChar() == '*') this.currentCharPos++;
        this.TokensList.add(currentToken);
    }



    /**
     * Adds a separator token or a paired-delimiter to the tokens list
     * @param op The current character.
     */
    private void AddSeparatorAndPunctuationToken(char op) {
        TokenType tokenType;

        // ( ) { } ; , [ ] : .
        if (op == '(') { tokenType = TokenType.OPEN_PARENTHESIS; }
        else if (op == ')') { tokenType = TokenType.CLOSE_PARENTHESIS; }
        else if (op == '{') { tokenType = TokenType.OPEN_CURLY_BRACES; }
        else if (op == '}') { tokenType = TokenType.CLOSE_CURLY_BRACES; }
        else if (op == ';') { tokenType = TokenType.SEMICOLON_SEPARATOR; }
        else if (op == ',') { tokenType = TokenType.COMMA_SEPARATOR; }
        else if (op == '[') { tokenType = TokenType.OPEN_SQUARE_BRAKET; }
        else if (op == ']') { tokenType = TokenType.CLOSE_SQUARE_BRAKET; }
        else if (op == ':') { tokenType = TokenType.COLON_SEPARATOR; }
        else if (op == '.' && hasNextChar() && nextChar() == '.') { tokenType = TokenType.RANGE_OPERATOR; }
        else { tokenType = TokenType.DOT_SEPARATOR; }

        String opText = (tokenType == TokenType.RANGE_OPERATOR) ? ".." : Character.toString(op);
        Token currentToken = new Token(tokenType, this.currentLineNum, this.currentCharPos, opText, null);
        this.TokensList.add(currentToken);

        // Accounts for the fact that the range operator takes two characters
        if (tokenType == TokenType.RANGE_OPERATOR) currentCharPos++;
    }



    /**
     * Adds a numeric literal to the tokens list
     * @param charSequence The current character
     */
    private void AddNumericLiteralToken(char[] charSequence) {
        final int start = this.currentCharPos;
        char currentChar;
        StringBuilder numString = new StringBuilder();
        boolean foundFloat = false;

        while (currentCharPos < currentLineLength) {
            currentChar = charSequence[this.currentCharPos];

            if (Character.isWhitespace(currentChar)) break;
            // Support for range operator.
            if (hasNextChar() && currentChar == '.' && nextChar() == '.') break;
            // Support for real numbers and numbers with underscores.
            if (!Regex.Match(currentChar, "[0-9]|_|\\.") || (currentChar == '.' && foundFloat)) break;
            if (currentChar == '.') foundFloat = true;

            numString.append(currentChar);
            currentCharPos++;
        }

        // Adjusts the currentCharPos for the Tokenizer's for-loop.
        currentCharPos--;

        // Removes the underscores.
        StringBuilder noUnderscores = new StringBuilder();
        for (char digit: numString.toString().toCharArray()) {
            if (digit == '_') continue;
            noUnderscores.append(digit);
        }

        // Converts the string to a number. If it has a floating point, it
        // will be kept as a double, otherwise, it will be casted to an int.
        double doubleVal = Double.parseDouble(noUnderscores.toString());

        // The number's representation in the language
        TokenType numType = (!foundFloat) ? TokenType.INTEGER_LITERAL : TokenType.REAL_LITERAL;

        Token currentToken;
        if (!foundFloat) {
            currentToken = new Token(numType, this.currentLineNum, start, noUnderscores.toString(), (int)doubleVal);
        } else {
            currentToken = new Token(numType, this.currentLineNum, start, noUnderscores.toString(), doubleVal);
        }
        this.TokensList.add(currentToken);
    }



    /**
     * Adds a string literal to the tokens list.
     * @param charSequence The current line
     */
    private void AddStringLiteralToken(char[] charSequence) {
        final int start = this.currentCharPos;
        char currentChar;
        StringBuilder stringLiteral = new StringBuilder();

        while (currentCharPos < currentLineLength) {
            currentChar = charSequence[this.currentCharPos];
            stringLiteral.append(currentChar);

            // Breaks the loop when it finds the closing quotes, and the found quotes are not escaped.
            if ((currentChar == '"' && currentCharPos != start) && this.prevChar() != '\\') break;

            currentCharPos++;
        }

        Token currentToken = new Token(TokenType.STRING_LITERAL, this.currentLineNum, start, stringLiteral.toString(),
                stringLiteral);
        this.TokensList.add(currentToken);
    }


    /**
     * Finds keywords or identifiers in the code.
     * If a word outside quotes is not a keyword, then it
     * must be an identifier created by the programmer.
     * @param charSequence The current line
     */
    private void AddKeywordsOrIdentifierTokens(char[] charSequence) {
        final int start = this.currentCharPos;
        char currentChar;
        StringBuilder keyword = new StringBuilder();


        while (currentCharPos < currentLineLength) {
            currentChar = charSequence[this.currentCharPos];

            if (Character.isWhitespace(currentChar) || isIllegalIdentifierChar(currentChar)) break;
            keyword.append(currentChar);

            currentCharPos++;
        }

        // Adjusts the currentCharPos for the Tokenizer's for-loop.
        currentCharPos--;

        Token currentToken;
        if (Token.Keywords.containsKey(keyword.toString())) {
            TokenType kwType = Token.Keywords.get(keyword.toString());
            currentToken = new Token(kwType, currentLineNum, start,keyword.toString(), null);
        } else {
            currentToken = new Token(TokenType.CUSTOM_IDENTIFIER, currentLineNum, start, keyword.toString(), null);
        }
        TokensList.add(currentToken);
    }
}
