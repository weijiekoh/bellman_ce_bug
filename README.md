# The bug

The proof which this binary generates is of a Poseidon circuit with 240
constraints, originally produced using `circom`. I'm trying to compile Plonk
proof support from the `bellman_ce` repository to wasm but the proof that's
generated is invalid. The `opening_at_z_omega_proof` value of the proof
differs between the `multicore` and `wasm` features.

If you compile the code using the `multicore` feature, however, the proof is
valid. `multicore` code won't run in browser wasm, though.

This repository demonstrates a bug in the [bellman] library (commit
`455480a2db44ecc0423785b295981074800913e6` in the `beta` branch).

## How to reproduce it

To see the bug in action, follow these steps:

```bash
git clone https://github.com/weijiekoh/bellman_ce_bug.git && \
cd bellman_ce_bug && \
cargo build --release
```

Run the executable:

```bash
./target/release/bellman_bug_demo
```

The output should end with:

```
proof.opening_at_z_omega_proof: G1(x=Fq(0x0488c95ff1846acf563139c7d068945b7011031bb5a7877b15f5f870cec32376), y=Fq(0x208a64c364fc2224856e321e330b7368ad533fdbcd7bd9aaf55b5a0c1d457ed8))
false
Proof is invalid
```

Next, edit `Cargo.toml` and change the following line:

```
bellman_ce = { git = "https://github.com/matter-labs/bellman", branch = "beta", default-features = false, features = [ "wasm", "plonk" ] }
```

to:

```
bellman_ce = { git = "https://github.com/matter-labs/bellman", branch = "beta", default-features = false, features = [ "multicore", "plonk" ] }
```

The `wasm` feature should be changed to `multicore`.

Recompile the binary and run it again:

```
cargo build --release && \
./target/release/bellman_bug_demo
```

The output should now end with:

```
proof.opening_at_z_omega_proof: G1(x=Fq(0x05a9a24328df520291c63cdfe58ed0e27db283ce9af20d90df677f4d938600cb), y=Fq(0x2aa2b74c71a72e73c61fb265cf2c975f1746aba6c05458c70d89ed8d7f680b7c))
true
Proof is valid
```
