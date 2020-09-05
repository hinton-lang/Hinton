// Hint Bucket File

// When a package.hb file is present in a project,
// the program can be run using the '$ hb run .' command.

// HB stands for 'Hint Bucket', and is Hinton Script's
// version of npm, but decentralized. All fields
// in this file are optional, but highly recommended.

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


// **** Permission Management & Security
// Hinton Script is secure by default.
// It is based on how Deno.lang permissions work. 
// By declaring permissions here the programmer
// does not have to specify them in the console.
#permissions:
- "--allow-read"
- "--allow-network"
- "--allow-write"


// **** Dependency Control
// Although it is not required, all the dependencies
// for a project should be specified in this section
// of the package.hb file for consistency and accuracy.
// Declaring dependencies in this way allows the programmer
// to import dependencies in the corresponding files in the
// following way: import { module1, module5 } from "#dependencies"
// This is similar to what Deno.lang provides, but with a twist.
#dependencies:
- { module1, module2 } from "https://url.to.package1"
- { module3, module4 } from "https://url.to.package2/version/1.2.2" // dependencies support versioning
- * as module5 from "https://url.to.package3"