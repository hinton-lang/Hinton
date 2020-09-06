# Sets
Hinton Script supports sets just like in Python.

Sets can be defined as follows:
```swift
let first_five_primes: Set<Int> = { 2, 3, 5, 7, 11 }
```

Like other collections, sets support `x in set`, `set.length()`, and `for x in set`. Being an unordered collection, sets do not record element position or order of insertion. Accordingly, sets do not support indexing, slicing, or other sequence-like behavior.

## The 'Set<>' Datatype
To define a set, a type must be provided within the angled brackets. Only one type is supported, and all the elements of the set must be the same type. For example:
```swift

let first_five_primes: Set<Int> = { 2, 3, 5, 7, 11 }
let employees: Set<String> = { "Karla", "Ivon", "Jake", "Tamiya" }
let irrational_constants: Set<Real> = { 3.14, 2.72, 1.62 }
```

## Sets support the following methods:
```swift
// Gets the size of the set s
s.size()

// Test whether the element e is in the set s
e in s
s.contains(e)

// Test whether every element in s is in t
s.isSubset(t)

// Test whether every element in t is in s
s.isSuperset(t)

// New set with elements from both s and t
s.union(t)

// New set with elements common to s and t
s.intersection(t)

// New set with elements in s but not in t
s.difference(t)

// New set with elements in either s or t but not both
s.sym_difference(t)

// New set with a shallow copy of the set s
s.copy()

// Adds the element e to the set s
s.add(e)

// Removes the element e from the set s
s.remove(e) // { "Carlos", "Tamiya" }

// Removes and returns an arbitrary element from s; raises KeyError if empty.
s.pop()

// Removes all elements from s
s.clear()
```