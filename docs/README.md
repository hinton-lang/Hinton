# Hinton Script

A general overview of what I imagine Hinton Script to be like.

## Primary features
- **Dynamically Compiled**: The programmer runs a program by typing `$ hinton run {filepath}` on the command line. When doing so, the program will be compiled to JVM byte code. After compilation, the program is not compiled again when running the same command on the same program. However, it is recompiled when any of the program files get modified.
- **Object-Oriented**: Hinton Script focuses on modularity, and brings the same system used by TypeScript for creating modules and packages. Any class, constant, or function can be explicitly exported from any program and imported into any other program. Also, everything in Hinton Script is an object.
- **Secure by Default**: Similar to Deno.js, Hinton Script provides built-in security. It requires certain flags if a program performs actions that may put the users at risk. Some of the flags are:
    - `--allow-network`: Grants permission to access the computer's network.
    - `--allow-read`: For accessing or reading any file from the file system.
    - `--allow-write`: For writing data and saving files into the file system.
- **Dynamically Typed**: In Hinton Script, types are inferred by default, but once a type is assigned to an identifier, it cannot be changed. The type system is very similar (if not the same) to the ones used by well established programming languages like TypeScript and Swift.
- **Feature Rich**: The goal of Hinton Script is to make the programmer's life easier, and unlike any other programming language, it actually does so. Hinton Script provides tons of methods attached to the built-in types, as well as many packages and modules with a great focus on A.I, Mathematics, Physics, and the likes.
- **Easy GitHub Integration**: The HB manager (Hinton Bucket Manager) provides an easy way of creating a project. Just type `$ hb new {project name} --git` on the console, and the command line will walk you through all the files and folders needed for a general application, as well as the ability to integrate GitHub into the project. The files generated include:
    - ./.git (folder)
    - ./.gitignore
    - ./README.md
    - ./package.yml
    - ./_hintBucket (folder)
    - ./main.ht
