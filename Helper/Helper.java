package org.hinton_lang.Helper;

public class Helper {
    /**
     * Converts the given object to a string for printing.
     *
     * @param object The object to tbe converted.
     * @return The string version of the object.
     */
    public static String stringify(Object object) {
        if (object == null)
            return "null";

        if (object instanceof Double) {
            String text = object.toString();
            if (text.endsWith(".0")) {
                text = text.substring(0, text.length() - 2);
            }
            return text;
        }

        return object.toString();
    }
}
