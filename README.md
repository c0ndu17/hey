# hey,

**hey,** is a self-organising peer-to-peer system where  
**Communication, using a stream of emergent identity.**

TLDR; The Internet 2: Electron Bugaloo 
    - Digital identity is fluid, where your interactions becomes the entropy that sculpts your identity.

- There are no IDs.  
- No configuration files.  
- No fixed ports.  
- No discovery service.  
- No membership protocol.

node begins from the same initial bytes(`ROOT`, aka. 'hey'). This enables the entropical introduction algorithm.
- In general, a systems entropy should be used and projected for entropy feedback

From that shared seed, each node evolves an internal state — called a `Node` — by folding in everything it experiences:

- whether it could bind to a port  
- messages it receives  
- data provided locally  
- the identity projections of other peers  

This evolving `Node` structure is more than state:

**It *is* the node’s identity,  
it *is* its history,  
and it *determines* how the node behaves next.**

A `Node` is either:

- a single bit:        0   or   1
- a pair of Nodes:     (Node, Node)

So, structurally:

    Node ::= Bit(0 | 1)
          |  Compound( Node , Node )

Visually:

          Compound
          /      \
       Node      Node


From this structure, each node deterministically derives:

- **its current network port**  
- **its next address** (inc. bind attempt outcome)  
- **how it interprets other nodes**  
- **how it communicates**  
- **how it participates in the mesh**

Collisions become information.  
Information becomes structure.  
Structure becomes behaviour.

Nodes don’t discover each other by asking “who is there?”  
They discover each other because **their deterministic behaviours converge and diverge in predictable ways**.

Over time, all nodes contribute to — and interpret — a shared symbolic dataspace:  
a small but powerful **symbol economy** where meaning emerges from entropy and interaction.

In one sentence:

> A network where everything — identity, topology, and communication — is reducible to an entropically shareable & decodable nodes/values.

---
## Running
To run **hey,**, you need to have [Rust](https://www.rust-lang.org/tools/install) installed.

Then, clone the repository and run any of the following commands in your terminal:

```
    ./hey,
    ./hey
    echo "hey" | cargo run
    cargo run --release
```

## Idea in One Sentence

A node’s identity is its value in a globally shared growing entropic space,
and each update nudges it toward the natural optimal balance point
of e^(1/e).


## GPT-5.1 Summary of some math
===


## Extra: Internal Coordinate Mapping via f, g, and H

This section outlines how atoms in the system are mapped into the bounded internal domain using the functions:

- \( f(x) = x^{e/x^e} \)
- \( g(x) = x^{e/x} \)
- Shared map \( H(u) = u\, e^{1-u} \)

### Properties of \(H\)

- Domain: \(u > 0\)  
- Bounded range with a single maximum at \(u = 1\)  
- \(H(1) = 1\)  
- \(H(u) \to 0\) as \(u \to 0^+\) or \(u \to +\infty\)

This makes \(H\) a **single-peaked bounded map**, suitable as an entropic potential.

---

### 3.1 Atom → Internal Coordinate via \(f, g, H\)

Each atom \(c\) is first mapped to a positive real code:

\[
x(c) \in \mathbb{R}^+
\]

Define two views:

**Internal view via \(f\):**
\[
u_{\text{int}}(c) = f(x(c)) = x(c)^{e/x(c)^e}
\]

**External-ish view via \(g\):**
\[
u_{\text{ext}}(c) = g(x(c)) = x(c)^{e/x(c)}
\]

Map both through the shared map \(H\):

\[
d_{\text{int}}(c) = H(u_{\text{int}}(c)), \qquad 
d_{\text{ext}}(c) = H(u_{\text{ext}}(c))
\]

Use:

- \( d_{\text{int}}(c) \) as the **canonical embedding** \( \psi(c) \in D \)
- \( d_{\text{ext}}(c) \) as a **shadow coordinate** related to reconstruction behavior  
  (e.g. measuring how many external forms map to this internal representation)

Define an entropic weight for each atom:

\[
w(c) = -\log H(u_{\text{ext}}(c))
\]

These weights contribute to the entropic accounting during aggregation.

---

### 3.2 Internal Superposition via \(H\)-Weighted Aggregation

Given atoms \(c_1, \ldots, c_k\), let:

- \( d_i = \psi(c_i) = d_{\text{int}}(c_i) \)  
- \( w_i = w(c_i) \)

Define the superposition operator:

\[
d_x = \frac{\sum_i d_i \, e^{-w_i}}{\sum_i e^{-w_i}}
\]

This yields:

- \( d_x \in D \), a **weighted barycenter** in the bounded domain  
- Heavier (more entropically significant) atoms influence the coordinate more strongly

Store:

- \( d_x \) as the **primary internal address** of the composed object
- The decomposition structure and weights in state \(S\), enabling reconstruction and sharing

**Deduplication rule:**  
If objects \(x\) and \(y\) contain the **same multiset of atoms** (up to multiplicity), the system produces:

\[
d_x = d_y
\]

This gives deterministic, number-theoretic deduplication.

---
