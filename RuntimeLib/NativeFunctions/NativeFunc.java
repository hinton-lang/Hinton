package org.hinton_lang.RuntimeLib.NativeFunctions;

import org.hinton_lang.Interpreter.HintonCallable;

/**
 * Signature interface for implementing native Hinton functions.
 */
public abstract interface NativeFunc {
    /**
     * Specifies the function's name.
     */
    public String getFuncName();

    /**
     * Specifies the function's body.
     */
    public HintonCallable getFunc();
}
