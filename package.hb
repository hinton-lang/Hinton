// Hint Bucket File

// When a package.hb file is present in a project,
// the program can be run using the '$ hb run .' command.

// HB stands for 'Hint Bucket', and is Hinton Script's
// version of npm. All fields in this file are optional,
// but highly recommended.

// **** Metafields
#name:          "Some Project"
#author:        "Some Author"
#version:       "1.0.0-rc1.2"
#licence:       "MIT"
#description:   "The project's description"
#contributors:
- "John Doe"
- "Marky Mark"
- "Jana Doe"
#email:         "admin-email-address@host.com"
#website:       "https://the-projects-website.com"


// **** Making the program an installable module
// If this field is provided, when other programmers
// install this package, the exported contents of
// file.hint will be available to the programmer.
#main:          "./path/to/main/file.hint"

// **** Executable scripts
// For example, to execute the 'start' script,
// type "$ hb run start" in the console.
#scripts:
- "start":      "hint helloworld.bbs"

// **** Compilation Output
// By default, when a program is compiled, the
// JVM byte code will be placed in a folder named
// '__output__' at the level as the 'package.hb' file.
// Changes the level of the output folder.
#outFolderPath: "../some/path/to/folder"
// Changes the default '__output__' folder name to be '_myOutput'.
#outFolderName: "_myOutput"


// **** Permission Management & Security
// Hinton Script is secure by default.
// It is based on how Deno.js permissions work. 
// By declaring permissions here the programmer
// does not have to specify them in the console.
#permissions:
- "--allow-read"
- "--allow-network"
- "--allow-write"


// **** Dependency Control
// All the dependencies for a project should be specified
// in this section of the package.ev file for consistency
// and accuracy. Packages will be installed inside the
// "_hintBucket" folder.
#dependencies:
- "some_package": "1.1.2"
- "someOtherPackage": "1.0.0-rc1.2.6"