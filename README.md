# Hinton

![Hinton Logo](_assets/logos/banner_light_mode.png#gh-light-mode-only)
![Hinton Logo](_assets/logos/banner_dark_mode.png#gh-dark-mode-only)

**The interpreted programming language for AI developers and Engineers.**

---

## The idea

Most numerical code today has two personalities. It looks like ordinary
dynamic code — then a framework traces one execution of it, captures the
operations it happened to see, and rebuilds the program as something else in
order to differentiate and compile it.

Everything painful lives on that boundary. Shapes are discovered too late.
Axis-order mistakes stay valid programs when the sizes happen to line up.
Ordinary control flow breaks graph capture. Some values must mysteriously be
"static" and others "traced". Custom operations fall out of the language into
C++ or CUDA. The code that runs stops being the code you wrote.

None of that is a defect in any one framework. It is the cost of rebuilding a
compiler's knowledge *around* a language that never exposed it.

Hinton starts from the other end: **the compiler understands Tensors well enough to check,
differentiate, place, and optimize them.** Every dimension has a name.
Differentiation is a transform over the program the compiler already holds. There is no eager/graph split, no export
dialect, and no second language for kernels.

Hinton keeps the familiar syntax and progressive typing style of popular interpreted programming languages, has a garbage
collector, and ships with a single binary for the entire toolchain. It is meant for real world applications without friction, including ML and AI development, physics and engineering simulations, and data science with heavy workflows.


## Tour of the Vision

```rust
freeze const HYPER = {"rate": 0.05, "batch": 64};

enum Trend { Improving, Plateaued, Diverging }

// Axes carry names, so `x @ w` contracts "feature" against "feature". Passing
// a ("batch", "token") tensor here is refused by name — even when every size
// happens to match, which would be a silent bug.
$[differentiable]
fn loss(
  w: Tensor<"f32", ("feature", "hidden")>,
  b: Tensor<"f32", ("hidden",)>,
  x: Tensor<"f32", ("batch", "feature")>,
  y: Tensor<"i64", ("batch",)>,
): Float {
  let scores = (x @ w + b).relu();
  return scores.cross_entropy(labels = y, over = "hidden");
}

// The compiler transforms `loss`'s own tree. Nothing is traced, so branches
// stay branches, loops stay loops, and an underivable line names itself.
let step = grad_of(loss, wrt = ["w", "b"]);

let w = tensor.randn([784, 128], axes = ["feature", "hidden"]);
let b = tensor.zeros([128], axes = ["hidden"]);
let history = [];

for (images, labels) in Tensor.load("mnist.npy").batches(size = HYPER["batch"]) {
  let grads = step(w, b, images, labels); // forward and backward, once
  w -= HYPER["rate"] * grads["w"];
  b -= HYPER["rate"] * grads["b"];
  history.push(grads.value); // the loss, already computed
}

fn trend(losses: Array<Float>): Trend {
  let [first, ..., last] = losses;
  
  return match last {
    case v if v > first put Trend.Diverging,
    case v if first - v < 0.01 put Trend.Plateaued,
    else put Trend.Improving,
  };
}

// Print the model's trend to the console
println(match trend(history) {
  case Trend.Improving put "converged at ${history[-1]:.4f}",
  case Trend.Plateaued put "stalled at ${history[-1]:.4f}",
  case Trend.Diverging put "diverging — lower the rate",
});
```

Note: Type Annotations are optional. The same program can run with no typing annotations and the axis names can still help catch common tensor operation mistakes, just at run time instead of compile time. However, each annotation added moves a check away from the VM or unlocks something the compiler can optimize.

### Status
Hinton is still in its early development stage. We are working on a separate private repository and version `0.1.0` is planned for public release in the coming months.

### License
Apache License 2.0. See [LICENSE](LICENSE).