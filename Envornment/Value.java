package org.hinton_lang.Envornment;

public class Value {
    private Object value;
    public final boolean isConst;

    public Value(Object value, boolean isConst, String type) {
        this.value = value;
        this.isConst = isConst;
    }

    public Value(Object value, boolean isConst) {
        this.value = value;
        this.isConst = isConst;
    }

    /**
     * Resets the value.
     * 
     * @param value The new value.
     */
    public void setValue(Object value) {
        if (!isConst) {
            this.value = value;
        }
    }

    /**
     * Gets the value.
     * 
     * @return The stored value.
     */
    public Object getValue() {
        return this.value;
    }
}
