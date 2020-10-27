package org.hinton_lang.AbstractSyntaxTree;

import java.util.ArrayList;

public class Compound extends AST {
   public ArrayList<AST> children;

   public Compound(ArrayList<AST> children) {
       this.children = children;
   }

    @Override
    public String toString() {
        return "Compound{" +
                "children=" + children +
                '}';
    }
}
