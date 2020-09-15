package com.wci.message;

public interface MessageProducer {

    /**
     * Add a listener to the listener list.
     * @param listener the listener to add.
     */
    public void addMessageListener(MessageListener listener);


    /**
     * Removes a listener from the listener list.
     * @param listener the listener to remove.
     */
    public void removeMessageListener(MessageListener listener);


    /**
     * Notify the listeners after setting the message.
     * @param message the message to set.
     */
    public void sendMessage(Message message);
}