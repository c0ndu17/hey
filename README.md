# hey,

`hey,` is a rust based cli that connects users through a shared entropic data space, in which everything is represented as a recursive binary structure.

[1, [(...)?, [1,(...)?]?]

The starting POC of a shared entropic data space.

The idea is after, `hey`, everything is within & projected through the shared data space. 

Instead of sending messages, each node in network evolves within the same bit-space.

Updates are folded into a recursive binary structure whose growth naturally
approaches the universal efficiency limit:

    e^(1/e) ≈ 1.4447

This value is the optimal balance point for branching,
information gain, and entropy-efficient decomposition.

And so beginneth the symbol economy.

---

## Core Aim

Transport of shared data frames, the result of repeated XOR, against e^(1/e).

The reason being is to base a network, on clear thermodynamic principles, 
- Using XOR, to split information over the network, into a client request, and server dispersal.
- XOR should be able to ensure the thermodynamic completeness, of XOR operations.

i.e. realtime entropic space through frequent UDP Packets, which reconstruct the space.

Each update transforms the node by:

```
    # x^(1/x) -> x^(e/x^e)
    next(state, input) => decode(encode(state) + encode(input))
```
Where:

- `encode(Node)` = recursive bitstring (fractal, self-similar)
- `decode(bits)` = re-factor into a balanced tree
- the conceptual leading `1` ensures monotonic growth, and change.

The bitstring always grows; the tree always re-balances.

---

## Continuity

Nodes do not address each other.

Continuity between nodes A and B is simply:

    Δ = encode(A) XOR encode(B)

Small Δ = close in entropic space  
Large Δ = diverged histories

This creates an undirected, shared field of state rather than a message graph.

---

## Running

    cargo run

Then type input or pipe data:

```
echo "hey" | cargo run
```

Each step prints:

- the external hex bitstring  
- the recursive structure  
- the 0/1 leaf distribution (an entropy sketch)

---

## Idea in One Sentence

A node’s identity is its position in a globally shared entropic space,
and each update nudges it toward the natural optimal balance point
of e^(1/e).
