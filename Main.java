package org.hinton_lang;


import org.hinton_lang.Parser.Parser;


public class Main {

    public static void main(String[] args) {
        String fPath = "/Users/faustotnc/Documents/GitHub/Hinton-Script/src/org/hinton_lang/test_file.ht";

        Parser parser = new Parser(fPath);
        System.out.println(parser.parse());
    }
}
