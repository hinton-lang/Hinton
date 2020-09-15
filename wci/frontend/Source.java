package com.wci.frontend;

import java.io.BufferedReader;
import java.io.IOException;

import com.wci.message.*;

public class Source implements MessageProducer {
    public static final char EOL = '\n';
    public static final char EOF = (char)0;

    private BufferedReader reader;
    private String line;
    private int lineNum;
    private int currentPos;
    

    /**
     * Constructor.
     * @param reader The reader for the source program.
     * @throws IOException if an I/O error occurred.
     */
    public Source(BufferedReader reader) throws IOException {
        this.lineNum = 0;
        this.currentPos = -2; // set to -2 to read the first source line
        this.reader = reader;
    }


    /**
     * Return the source character at the current position.
     * @return the source character at the current position.
     * @throws Exception if an error occurred.
     */
    public char currentChar() throws Exception {
        // first time?
        if (currentPos == -2) {
            readLine();
            return nextChar();
        }

        // At end of file?
        else if (line == null) {
            return EOF;
        }

        // At end of line?
        else if ((currentPos == -1) || currentPos == line.length()) {
            return EOL;
        }

        // Need to read next line?
        else if (currentPos > line.length()) {
            readLine();
            return nextChar();
        }

        // Return the character at the current position.
        else {
            return line.charAt(currentPos);
        }
    }


    /**
     * Consume the current source character and return the next character.
     * @return the next source character.
     * @throws Exception if an error occurred.
     */
    public char nextChar() throws Exception {
        ++currentPos;
        return nextChar();
    }


    /**
     * Return the source character following the current character without
     * consuming the current character.
     * @return the following character.
     * @throws Exception if an error occurred.
     */
    public char peekChar() throws Exception {
        currentChar();
        if (line == null) return EOF;
        int nextPos = currentPos + 1;
        return nextPos < line.length() ? line.charAt(nextPos) : EOL;
    }


    /**
     * Read the next source line.
     * @throws IOException if and I/O error occurred.
     */
    private void readLine() throws IOException {
        line = reader.readLine(); // null when at the end of the source
        currentPos = 0;
        if (line != null) ++lineNum;

        // Send a source line message containing the line number
        // and the line text to all the listeners.

        if (line != null) {
//            sendMessage(new Meesage)
        }
    }


    /**
     * Close the source.
     * @throws Exception if an error occurred.
     */
    private void close() throws Exception {
        if (reader != null) {
            try {
                reader.close();
            } catch(IOException ex) {
                ex.printStackTrace();
                throw ex;
            }
        }
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
