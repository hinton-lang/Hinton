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
let employees = Set<String> = { "Karla", "Ivon", "Jake", "Tamiya" }
let engineers = Set<String> = { "Tamiya", "Ivon" }

// Gets the size of the set
employees.size() // 4

// Test whether every element in engineers is in employees
engineers.isSubset(employees) // true
employees.isSubset(engineers) // false

// Test whether every element in employees is in engineers
engineers.isSuperset(employees) // false

// New set with elements from both engineers and some_names
let some_names = { "Carlos", "Tamiya", "James" }
engineers.union(some_names) // { "Carlos", "James", "Ivon", "Tamiya" }

// New set with elements common to engineers and some_names
engineers.intersection(some_names) // { "Tamiya" }

// New set with elements in employees but not in engineers
employees.difference(engineers) // { "Karla", "Jake" }

// New set with elements in either engineers or some_names but not both
engineers.symmetric_difference(some_names) // { "Carlos", "Ivon", "James" }

// New set with a shallow copy of employees
employees.copy() // { "Karla", "Ivon", "Jake", "Tamiya" }

// Adds an element to employees
employees.add("Tashir") // { "Karla", "Ivon", "Jake", "Tamiya", "Tashir" }

// Removes an element from some_names
some_names.remove("James") // { "Carlos", "Tamiya" }

// Removes and returns an arbitrary element from employees; raises KeyError if empty.
// For the purposes of this example, let's assume that the element "Jake" was removed from the set.
employees.pop() // "James"

// Removes all elements from engineers
engineers.clear()
```