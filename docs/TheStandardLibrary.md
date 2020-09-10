# Hinton Script's Standard Library.


- **Binary Trees**: A class that allows the creation of deeply nested binary trees.
```swift
import { BinaryTree, BinaryNode, Leaf } from 'stdlib@ADT`

let myTree = new BinaryTree(value = 11);

myTree.left = new BinaryNode(value = 22);
myTree.right = new Leaf(2);

myTree.left.left = new Leaf(66);
myTree.left.right = new Leaf(3);

// The above code generates the following tree
//         11
//         /\
//        /  \
//       /    \
//      22     2
//      /\     
//     /  \
//    /    \
//   66     3
```