package com.wci.frontend;

import com.wci.Intermediate.ICode;
import com.wci.Intermediate.SymTab;

import com.wci.message.MessageHandler;
import com.wci.message.MessageProducer;
import com.wci.message.MessageListener;
import com.wci.message.Message;


public abstract class Parser implements MessageProducer {
    protected Scanner scanner;
    protected ICode icode;
    protected static SymTab symTab; // generated symbol table
    protected static MessageHandler messageHandler; // manage handler delagate

    static {
        symTab = null;
        messageHandler = new MessageHandler();
    }


    /**
     * Constructor.
     * @param scanner the scanner to be used with this parser.
     */
    protected Parser(Scanner scanner) {
        this.scanner = scanner;
        this.icode = null;
    }


    /**
     * Parse a source program and generate the intermediate code and the
     * symbol table.
     * To be implemented by a language-specific parser subclass.
     * @throws Exception if an error occurred.
     */
    public abstract void Parse() throws Exception;


    /**
     * Return the number of syntax errors found by the parser.
     * To be implemented by a language-specific parser subclass.
     * @return the error count.
     */
    public abstract int getErrorCount();


    /**
     * Call the scanner's currentToken() method.
     * @return the current token.
     */
    public Token currentToken() {
        return scanner.currentToken();
    }


    /**
     * Call the scanner's nextToken() method.
     * @return the next token.
     * @throws Exception if an error occurred.
     */
    public Token nextToken() throws Exception {
        return scanner.getNextToken();
    }


    /**
     * Add a parser message listener.
     * @param listener the message listener to add.
     */
    public void addMessageListener(MessageListener listener) {
        messageHandler.addMessageListener(listener);
    };


    /**
     * Removes a parser message listener.
     * @param listener the message listener to remove.
     */
    public void removeMessageListener(MessageListener listener) {
        messageHandler.removeMessageListener(listener);
    };


    /**
     * Notify the listeners after setting the message.
     * @param message the message to set.
     */
    public void sendMessage(Message message) {
        messageHandler.sendMessage(message);
    };
}