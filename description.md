# NOVA — Language Description

## What is NOVA?

NOVA is a compiled, statically typed, general-purpose programming language built for the next generation of computing. It can build anything — web servers, CLIs, compilers, simulations, and data pipelines. But it was designed from the ground up by and for people who work at the intersection of science, artificial intelligence, data, and space. In NOVA, those things are not library features or plugins. They are the language.

NOVA is for the researcher training a transformer model on spectroscopic survey data. The engineer calculating delta-v budgets for a lunar insertion burn. The data scientist fitting a power law to the stellar mass-luminosity relation. The cosmologist estimating the Hubble constant from a Type Ia supernova catalogue. The student learning that `1.0[kg] + 1.0[m]` is not just wrong numerically — it is wrong *dimensionally*, and the compiler can prove it.

NOVA is general-purpose the way a spacecraft is general-purpose: capable of many things, optimised around a specific kind of excellence, and built with the understanding that the problems worth solving are rarely small.

---

## Design philosophy

### 1. Mission-first clarity

In NOVA, top-level functions are called **missions**. Modules are called **constellations**. Imports are called **absorb**. Output is called **transmit**. This is not decoration. It is a deliberate choice about what kind of programmer NOVA is speaking to, and what kind of code NOVA encourages them to write.

A NOVA program reads like a research paper with executable sections. Every mission has a name that describes its purpose, inputs with types and units, and a return type. The code inside is a sequence of named steps, each building on the last. When you read a NOVA program, you understand what it is *trying to do* — not just what it happens to compute.

```nova
mission analyze_main_sequence(catalog: Wave) → Void {
  let stars =
    read_fits(catalog)
    |> pipeline [
        filter(luminosity_class == "V"),
        map(s → { mass: s.mass_solar, lum: log10(s.luminosity_solar) }),
        drop_outliers(sigma: 3.0),
      ]
  let r              = pearson(stars.mass, stars.lum)
  let (slope, icept) = linear_fit(stars.mass, stars.lum)
  transmit("Pearson r = {r:.4}  |  log(L) = {slope:.3}·M + {icept:.3}")
  scatter(stars, x: mass, y: lum, color_by: surface_temp,
          title: "Main sequence: mass vs log luminosity")
  regression_line(slope, icept)
}
```

This reads like a description of the analysis. The code *is* the paper.

### 2. Units as types

Every numeric literal in NOVA can carry a physical unit, and that unit is part of the type at compile time. A value of type `Float[m/s]` is a velocity. A value of type `Float[kg]` is a mass. Attempting to add them is a type error — caught before any program runs, just like adding an integer to a string.

Unit arithmetic follows SI dimensional analysis automatically. Multiplying `Float[m/s²]` by `Float[kg]` produces `Float[N]`. The compiler knows that `kg·m/s²` is a Newton. Dividing `Float[m]` by `Float[s]` produces `Float[m/s]`. The full SI dimension vector (length, mass, time, current, temperature, amount, luminosity) is tracked for every value.

This is not a convenience feature. It is a safety guarantee. Unit errors have caused satellite losses, experimental failures, and years of wasted computation. NOVA makes them impossible at runtime by making them visible at compile time.

```nova
let isp        = thrust / (mass_flow * 9.80665[m/s²])   -- Float[s] — inferred
let period_yr  = 1.0[AU] |> kepler_period(m_star: 1.989e30[kg]) as [yr]
let wrong      = 1.0[kg] + 1.0[m]
-- Error: cannot add Float[kg] and Float[m]
--        dimension [M] + [L] — incompatible
```

### 3. AI-native

Tensors, gradients, and model training are first-class constructs in NOVA. There is no framework to install, no computation graph to build manually, no boilerplate to write.

The `tensor` type is built in. The `model` and `layer` keywords define neural architectures. The `autodiff` keyword triggers automatic differentiation. The `gradient ... wrt` expression computes gradients explicitly. Common operations — matmul (`@`), `softmax`, `relu`, `gelu`, `einsum`, `norm`, `reshape`, `cross_entropy`, `mse` — are part of the language.

```nova
model StarClassifier {
  layer conv1d(64, kernel=7, activation=.relu)
  layer conv1d(128, kernel=5, activation=.relu)
  layer pool(global_avg)
  layer dense(7, activation=.softmax)
}

parallel mission train(net: StarClassifier, data: Dataset[Tensor[Float, 4096], Int]) → StarClassifier {
  for batch in data.batches(32) {
    let loss = cross_entropy(net.forward(batch.x), batch.y)
    autodiff(loss) { net.update(adam, lr=3e-4) }
  }
  return net
}
```

No `import torch`. No `import numpy`. No session. No graph. Just the computation.

### 4. Parallel by default

Scientific computation is parallel by nature. A `parallel mission` automatically distributes its work across all available CPU cores using a work-stealing scheduler. Operations on `Array` and `Tensor` inside a parallel context are automatically vectorised. The programmer does not manage threads, locks, mutexes, or pools.

```nova
parallel mission process_survey(obs: Array[Spectrum]) → Array[EmissionLine] {
  return obs
    |> pipeline [
        filter(snr > 10.0),
        map(wavelength_calibrate),
        map(identify_emission_lines),
      ]
}
```

The compiler infers data independence, parallelises the pipeline stages that can run concurrently, and sequences those that cannot. The programmer states the *what*, not the *how*.

### 5. Strongly typed with inference

NOVA uses Hindley-Milner type inference extended with unit dimension vectors and tensor shape types. The programmer rarely writes type annotations — they are inferred from context. But the types are always present, always checked, and always include the unit.

Type errors are written for scientists and engineers, not compiler engineers. A unit mismatch names the physical dimensions that conflict. A shape mismatch prints both tensor shapes and the operation that failed. An ambiguous inference asks a specific question, not a generic one.

### 6. General-purpose

NOVA is not a domain-specific language. It is a general-purpose language that happens to be excellent at science and AI. It has a full standard library (`nova.*`) for networking, file systems, CLI tooling, databases, serialisation, and concurrency. A NOVA program can be a web server, a command-line tool, a data pipeline, or a compiled library callable from C.

The scientific standard library (`cosmos.*`) sits alongside this — not above it — as a curated set of constellations for the domains NOVA was inspired by.

---

## Syntax reference

### Variables

```nova
let mass     = 5.972e24[kg]          -- immutable, unit inferred
let velocity : Float[m/s] = 0.0[m/s] -- immutable, explicit type
var altitude = 400.0[km]             -- mutable
```

### Missions (functions)

```nova
-- Basic mission
mission escape_velocity(mass: Float[kg], radius: Float[m]) → Float[m/s] {
  let G = 6.674e-11[N·m²/kg²]
  return sqrt(2.0 * G * mass / radius)
}

-- Parallel mission — distributed across all cores
parallel mission reduce(data: Array[Spectrum]) → Array[Line] {
  return data |> pipeline [filter(snr > 5.0), map(identify)]
}

-- Mission with named argument call sites (all arguments can be named)
let dv = delta_v(isp=311.0[s], m_wet=549054.0[kg], m_dry=25600.0[kg])
```

### Constellations (modules)

```nova
constellation Orbital {
  export mission period(m: Float[kg], a: Float[AU]) → Float[yr] { ... }
  export let G = 6.674e-11[N·m²/kg²]
}

absorb Orbital.{ period, G }
absorb cosmos.stats.{ pearson, linear_fit, spearman }
```

### `pipeline [...]`

A pipeline block applies a list of named transforms in order. Comma-separated. Each step takes the output of the previous as its first argument.

```nova
let result =
  source
  |> pipeline [
      filter(condition),
      map(transform),
      drop_outliers(sigma: 3.0),
      sort_by(field),
    ]
```

### Pattern matching

```nova
match detection {
  Some(d) if d.snr > 12.0 => confirm(d)
  Some(d)                  => flag_for_review(d)
  None                     => log_missing()
}
```

### String interpolation

```nova
transmit("r = {r:.4}  v = {v as [km/s]:.2} km/s  n = {n}")
```

### Model and autodiff

```nova
model PhysicsNet {
  layer dense(512, activation=.relu)
  layer dense(256, activation=.relu, dropout=0.2)
  layer dense(1)
}

-- Automatic differentiation
let loss = mse(net.forward(x), y)
autodiff(loss) { net.update(adam, lr=1e-3) }

-- Explicit gradient
let grads = gradient(loss, wrt net.params)
```

### Custom units

```nova
unit parsec    = 3.086e16[m]
unit solar_lum = 3.828e26[W]
unit earth_rad = 6.371e6[m]

let d : Float[parsec] = 1.0[parsec]
let d_m = d as [m]   -- 3.086e16[m]
```

---

## Relationship to existing languages

| Language | What NOVA inherits | What NOVA does differently |
|---|---|---|
| Python | Readability, scientific community | Compiled, statically typed, unit-typed, parallel by default |
| Julia | Scientific focus, high performance, JIT | Unit types are compile-time (zero runtime cost), AI is zero-import |
| Rust | Performance, type system, memory safety | Scientist-friendly memory model, unit system, AI primitives built in |
| Mojo | AI-native, Python superset path | Full unit type system, constellation modules, mission syntax, general-purpose from day one |
| F# | Pipeline operator, units of measure, inference | Deeper SI unit system, tensor types, AI-native, `cosmos.*` standard library |
| Swift | Clean syntax, type inference, safety | Scientific focus, unit system, no mobile target |
| APL/J | Terse array operations | Readable, not terse; named everything; unit-typed arrays |

NOVA's unit type system is most directly inspired by F#'s units of measure, extended to the full 7-dimensional SI basis and integrated with tensor shape types and automatic differentiation.

---

## What NOVA is for — domain by domain

### Rocket science and aerospace

NOVA was partially inspired by the precision requirements of aerospace engineering. A rocket simulation that confuses `kg` and `N` can fail. A guidance algorithm that treats `m/s` as `m/s²` can crash. NOVA's unit type system makes these confusions impossible.

```nova
mission delta_v(isp: Float[s], m_wet: Float[kg], m_dry: Float[kg]) → Float[m/s] {
  return isp * 9.80665[m/s²] * ln(m_wet / m_dry)
}
```

The `cosmos.orbital` constellation provides: Tsiolkovsky's equation, Kepler's laws, orbital period, semi-major axis, eccentricity, trajectory integration, Hohmann transfer, and gravitational parameter calculations. All unit-typed.

### Astronomy and cosmology

NOVA reads FITS files natively via `cosmos.astro`. It handles the Hertzsprung-Russell diagram, spectral classification, photometric redshift, and cosmological distance calculations as standard library functions. The type system knows that `Float[M☉]` is a stellar mass and `Float[L☉]` is a luminosity — not arbitrary floats.

```nova
absorb cosmos.astro.{ read_fits, parallax_distance }
absorb cosmos.spectral.{ blackbody_peak, wien_displacement }

mission stellar_temperature(wavelength_peak: Float[nm]) → Float[K] {
  return wien_displacement(wavelength_peak)
}
```

### Data science and machine learning

`cosmos.ml` provides tensors, layers, optimisers, losses, and metrics as first-class language constructs. `cosmos.stats` provides the statistical machinery — distributions, tests, fits, Bayesian inference. `cosmos.data` handles the I/O: CSV, Parquet, FITS, HDF5, Arrow.

The `pipeline [...]` construct is designed specifically for data science workflows: a clean, readable chain of filtering, transforming, and aggregating steps, parallelised automatically.

### Next-generation AI

NOVA is designed for the researcher building the next generation of AI, not just applying the current generation. It supports large model definitions, efficient training loops with automatic differentiation, and custom layer types. Because units propagate through tensor operations, a physics-informed neural network in NOVA carries physical meaning at the type level.

```nova
model NeuralODE {
  -- Learns a differential equation from data
  layer dense(256, activation=.tanh)
  layer dense(256, activation=.tanh)
  layer dense(state_dim)
}

mission dynamics(net: NeuralODE, state: Tensor[Float[m/s], 6], t: Float[s]) → Tensor[Float[m/s²], 6] {
  return net.forward(concat(state, [t]))
}
```

The return type `Tensor[Float[m/s²], 6]` is an acceleration vector — physically typed. Passing it where a velocity is expected is a compile-time error.

---

## The name

NOVA. A nova is a stellar event in which a white dwarf in a binary system accretes material from its companion star until a thermonuclear runaway ignites on its surface — a sudden, dramatic increase in luminosity. Energy that has been accumulating suddenly releases. What was dim becomes brilliant.

The name reflects the language's purpose and its inspiration: to release the energy that has been accumulating in scientific computing, compressed between the expressiveness of Python and the performance of compiled code. To make computation in science and AI faster, clearer, and more powerful than what came before. To illuminate what was dark.

The file extension is `.nv`.
