# mpst-inference
 Simple inference experiment for session types in Rust

## Setup

Make sure `cargo` is installed. 

## Tests

Run `cargo run test` under the main directory if you want all tests to be run, or under the specific crate you want to test (`inference` being the most interesting).

## Project Structure

### `session/`

This defines and implements local session types, `LocalType/PartialLocalType` which are used when inferring session types from Rust code, and `MPSTLocalType` which is closer to the Local Session Type definition from [A Very Gentle Introduction to Multiparty Session Types](https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=&cad=rja&uact=8&ved=2ahUKEwi-jP-R7YeEAxUpU0EAHS6jDhEQFnoECA4QAQ&url=http%3A%2F%2Fmrg.doc.ic.ac.uk%2Fpublications%2Fa-very-gentle-introduction-to-multiparty-session-types%2Fmain.pdf&usg=AOvVaw360ekX9Vth4pifImS63Nkg&opi=89978449).

Participants are also defined here, with the rest of the crate relying on the assumptions that participants are anonymous (or unspecified) until the merging of local types.

### `macros/`

This proc-macro crate exports `infer_session_type`, an attribute macro which generates a function that returns the inferred local type for a given function (representing a standalone program, or participant in a protocol) that operates on an input session. The macro works as follows:
```rust
#[macros::infer_session_type]
fn some_program(mut s: Session) {
    ...
}

// Macro defines the following:
// fn get_session_type_some_program() -> LocalType
// fn get_mpst_session_type_some_program() -> MPSTLocalType
```

It might be better to instead define static values, but this is more of an ergonomic choice rather than a technical limitation.

### `inference/`

This crate contains all the code used for merging local types into a (potentially) compatible global one. It defines `merge_locals`, which takes as an input a vector of participant names and their MPST local types, and returns a `GlobalType` from the merging algorithm. The merging algorithm is described below.

#### Merging algorithm

The algorithm is a recursive operation on an input set of local types keyed by their participant names. The main operation is as follows:
1. (End-Termination) If the product of local types is already `End x End x ... x End`, then terminate and return `GlobalType::End`
2. (Dual-reduction) Otherwise, enumerate the corresponding local types of the dual, and for each dual, synthesise a "step" (Select) in the Global type, and then recurse with the resulting set of local types (Goto step 1 with the dual-reduced system). The first dual-reduced recursion that returns a valid GlobalType is then used as the continuation, and terminates.
3. If no dual-reduction is possible, then we might need to handle a recursive declaration or call.
- (Recursion unwrap) If 1 or more LTs are a recursive declaration, then generate a recursive declaration in the global type (mapping the corresponding local recursive calls to the new global recursion ID), and call the main algorithm on the set of local types with the outer recursive declaration removed (Goto step 1 with the rec-unwrapped local types).
- (Recursion call matching) Otherwise, ensure that any recursive calls point to the same global recursion ID (breaks completeness, see counter-example 1), and assume any LT that is not a recursive call is compatible with the expanded recursion (recursion prefix). (TODO: Check that the expansion-then-reduction of the LTs leads to a cycle, which indicates compatibility). If true, then simply merge into a recursion to the specified global recursion ID. Otherwise, error out with the problem behaviour.

##### Completeness

The algorithm is complete without the global recursion identity check, and correct without the recursion prefix assumption.

(Where is the proof?)

