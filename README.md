# numeric-domains: abstract domains for fixed-width numeric values

Experimental Rust implementations of several abstract domains for 64-bit machine integers:

- `Tnum`: known-value/unknown-mask tracking numbers. Arithmetic wraps at 64 bits and shift counts
  are reduced modulo 64.
- `Znum`: tracks whether each bit may be zero and/or one. Addition wraps at 64 bits and shift
  counts are reduced modulo 64.
- `Rnum`: a range representation with independent signed and unsigned bounds.

All three domains provide constant and membership queries, extrema, and signed/unsigned bound
inspection. They also provide `union` (the least representable domain containing both operands) and
`intersection` (the values common to both operands), including empty intersections. `Tnum` and
`Znum` additionally support bitwise operations and wrapping addition.

This crate is experimental and is not yet published as a stable API.

# Misc

 - https://en.wikipedia.org/wiki/Abstract_interpretation

 - "Pentagons: A Weakly Relational Abstract Domain for the Efficient Validation of Array Accesses"
   - "more precise than Interval, less preciece than Octogon"
   - https://www.microsoft.com/en-us/research/wp-content/uploads/2009/01/pentagons.pdf


 - http://research.cs.wisc.edu/wpis/papers/vmcai17.pdf
 - http://bitmath.blogspot.com/2013/08/addition-in-bitfield-domain.html
 - http://bitmath.blogspot.com/2014/02/addition-in-bitfield-domain-alternative.html
 - "Abstract Domains for Bit-Level Machine Integer and Floating-point Operations"
 - https://www.omnimaga.org/other-computer-languages-help/addition-in-the-bitfield-domain/
