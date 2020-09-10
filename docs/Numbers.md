# Numbers and the Different Numeric Data-types
Hinton Script supports three primary data-types for representing numbers, which correlate with mathematical concepts.

## Integers
Integers are represented by the type `Int`, and the data it stores is equivalent to Java's `long int` data type. Later in the project, the goal is to make it so that integers have not bound max and min values.

### Integer methods
```swift
let n: Int = 432;

// Returns the integer casted to a string.
n.toString();

// `True` if the integer is even, `false` otherwise.
n.isEven();

// `True` if the integer is odd, `false` otherwise.
n.isOdd();

// ***** REVISE: THIS NEEDS A BETTER NAME
// The sum of all the values in the returned
// array equals the original integer.
n.places(); // [2, 30, 400]

// Returns an array containing the digits
// of the original integer.
n.digits() // [4, 3, 2]
```


## Reals
Real numbers are represented by the type `Real`, and the data they store is equivalent to Java's `double` data type.

### Real Methods
```swift
let r: Real = 32.3435563;

// Returns the real number casted to a string.
r.toString();

// Returns the integer part of the real number
r.getInt(); // 32

// ***** REVISE: THIS NEEDS A BETTER NAME
// Returns the original number minus the
// integer part.
r.getDecimals(); // 0.3435563

// Returns the number of decimal places after the period.
n.precision(); // 7
```


## Complex
Represented by the type `Complex`, and the data is stored in rectangular form as a 2-tuple with both entries being a real-valued number. The first entry of this tuple is the real part of the complex number, while the second entry of the tuple is the imaginary part of the complex number.

Complex numbers in Hinton Script are written the same way they are written in mathematics:
```swift
let z = z.real + z.imaginary*1i;
```

### Complex Methods
```swift
let c: Complex = 2 + 3.4i;

c.toString();

// Returns the real part of the complex number
c.real();

// Returns the imaginary part (of type Real)
// of the complex number.
c.imaginary();

// Converts the complex number to polar.
c.toPolar();

// Converts the complex number to rectangular.
c.toRectangular();

// Returns the distance of the complex number
// to the origin of the x-complex plane.
c.length();

// Returns the angle the complex number
// makes with the x-axis.
c.angle();

// ***** EXPERIMENTAL
// If added to the final version, this methods
// will provide a map from complex numbers to
// a color plane.
c.color();
```