/** # Soundness bugs in Rust libraries: can't live with 'em, can't live without 'em

<sup>*by [David Tolnay]&#8202;,&ensp;2019.12.09*</sup>

[David Tolnay]: https://github.com/dtolnay

<br>

My role at $work these days is to help guide a big company's investment in Rust
toward success. This essay covers a slice of my experience as it pertains to
unsafe code, and especially bugs in unsafe code.

<br>

## An appropriate mindset for this discussion

Rust is strikingly and truthfully marketed as a **safe** language. More than its
memory safety and thread safety guarantees, the language exposes facilities to
library designers for building abstractions that resist misuse. The emergent
safe library ecosystem enables "if it compiles, then it's correct" programming
unmatched by other mainstream languages, even garbage collected ones.

Rust is a **performant** language, which to some is a convenient bonus while to
others it's table stakes for many interesting use cases.

Safety and performance could be gotten decades ago by writing formal
mathematical proofs and Rust is not that. Rust brings safety and performance in
a **productive** modern language so that we can iterate and ship things.

But an asterisk to all three above qualities is that Rust is a practical
language. Tradeoffs exist. Perfect safety is unrealistic. The systems we build
in Rust will run on real hardware whose circuits Rust can't prove are correct,
run on real operating systems whose bugs Rust won't isolate you from, and often
embed fragments of other languages that Rust has no visibility into.

It is tempting when discussing unsafe Rust to feel that the whole enterprise is
for nothing, that if unsafe makes it possible to have C++-style memory safety
bugs then there's no point to Rust and we might as well continue writing in C++.
When facing this mindset, it helps to imagine a choice among the following:

1. modern C++
2. a language many times safer than modern C++
3. an imaginary language, infinitely safer than modern C++ but nonexistent

We can argue about what the multiplier between #1 and #2 could be (and the rest
of the essay will shed some light on this), but it's clear that a value
substantially less than infinity is sufficient to make #2 a worthwhile choice
for building real systems.

<br>

## Soundness

Soundness is a statement about whether *all possible uses* of a library or
language feature uphold the intended invariants. In other words it describes
functionality that cannot be misused, neither by mistake nor maliciously.

It is worth internalizing this understanding of soundness when evaluating
soundness bugs; they are a very different sort of bug than typical exploitable
memory safety vulnerabilities like use-after-free or buffer overflows. When a
library is unsound, it tells you the library is possible to misuse in a way that
could be a vulnerability, but it does not tell you that any code has already
misused the library in such a way.

In my experience discovering unsound library code in my work codebase, so far
it's always only been hypothetical contrived code that could be broken; the
existing uses of the unsound libraries have always been correct. We fix the
soundness bugs to ensure it remains that way as the codebase scales.

<br>

## Simple case study

To drive home this view of soundness and give a first look at unsound Rust
library code, consider a C function that we want to make callable from Rust.

```
# const IGNORE: &str = stringify! {
// Requires arg != 10.
// If arg is 10, we stomp on yer memery.
void frob(int32_t arg);
# };
```

An impractical safe language might decide that we just don't support calling C. 
Any C code can potentially do whatever in a way that is not visible to our safe
language's compiler, so the only way to uphold any meaningful safety guarantee
on the whole program is by forbidding calling C.

A different impractical language might allow calling C but give up on safety
guarantees on any code that transitively does so; safety guarantees would only
apply to code written purely in the safe language. This is next to useless
because in practice only a small fraction of a real program would benefit.
Anything that involves a memory allocator (strings, vectors) or system call
(reading a file) would be impossible to define in a way that resists misuse.

In designing a **practical** safe language we look for ways to make safety
guarantees about as much of the program as possible subject to those guarantees
being as useful as possible *in practice*. We enforce that the tiny fraction of
code in which the programmer takes responsibility for maintaining invariants are
demarcated and we audit them.

One safe way to bind the `frob` function above would be by introducing runtime
validation of the argument. The following binding is safe for the caller to call
because no possible argument they can pass can lead to violation of invariants.
During an audit we can find this unsafe block, read these few lines and the
documentation of C frob, and be confident that the system is sound.

```
# mod ffi {
#     pub unsafe fn frob(_arg: i32) {}
# }
#
pub fn frob(arg: i32) {
    assert!(arg != 10);
    unsafe { ffi::frob(arg) }
}
```

Soundness does not always imply runtime validation. Most of the time we can
leverage Rust's ownership rules, move semantics, lifetimes, and other language
facilities to design auditable safe abstractions around unsafe code at zero
runtime cost. For example perhaps the `frob` argument is expected to be one of a
limited set of values that we can represent by a Rust enum:

```
# mod ffi {
#     pub unsafe fn frob(_arg: i32) {}
# }
#
pub enum FrobLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

pub fn frob(level: FrobLevel) {
    let arg = level as i32;
    unsafe { ffi::frob(arg) }
}
```

As a last resort we sometimes pass on responsibility for safety invariants to
the caller in cases that cannot be enforced in a low level library.

```
# mod ffi {
#     pub unsafe fn frob(_arg: i32) {}
# }
#
// Safety: caller must ensure arg != 10.
pub unsafe fn frob_unchecked(arg: i32) {
    ffi::frob(arg)
}
```

But what if someone were to write the following binding? Then this library is
unsound. This binding is possible to invoke in a way that leads to memory
unsafety, and Rust will not stop you because it does not understand the
documentation on your C function.

```
# mod ffi {
#     pub unsafe fn frob(_arg: i32) {}
# }
#
pub fn frob(arg: i32) {
    // UNSOUND
    unsafe { ffi::frob(arg) }
}
```

But despite the unsoundness, it is important to recognize that no undefined
behavior or vulnerability necessarily exists. If the only frob call in our
codebase is `frob(0)`, it may not even be such a high priority to address the
soundness bug.

<br>

## Unsoundness as a reflection of priorities

Some projects begin in a mode where unsound library code is not a big deal. In
the Zero to One phase of a project where all we want is to demonstrate that a
concept is viable, what use is painstakingly designing safe abstractions while
the deadlines fly by? Recall that unsoundness does not mean that your software
is broken or has undefined behavior. Soundness of a library is a statement about
all possible uses, but if the two uses in your project today are fine then you
likely have more immediate priorities to deal with.

To be clear, not all projects and not all companies would permit a mode like
this. Some would prefer to build the thing correctly from the beginning or not
at all, which is my personal style as well. Fundamentally there is a
latency/throughput tradeoff involved as with any technical debt: tolerating
unsoundness can be seen as a latency optimization, getting something working
sooner but having to revisit and redesign later in a way that wouldn't be
necessary if good abstractions were in place all along.

As my employer ramps up more and more projects and engineers in Rust, it falls
on me to mitigate the dominant engineering culture and effect a culture change
toward caring about "all possible uses" of core libraries. A sloppy unsound
library from long before I joined could have been a practical justifiable
tradeoff at the time, but with dramatically more users it becomes inevitable
that it will be misused and cause vulnerabilities. I have made it part of my job
to shore up a core of foundational library abstractions that I personally
guarantee are sound.

Lastly, keep in mind that the calculus on unsoundness can be a bit different
between an industry monorepo codebase and an open source library. Everything on
this page is from the industry point of view where we have perfect visibility
into all callers of a library for analysis. On the other hand unsoundness in the
public API of a third party project is a huge red flag that must not be
normalized, and is almost guaranteed to disqualify a library for our purposes.
An open source library maintainer cannot have visibility into all uses and thus
must treat any unsoundness as if it were causing high priority vulnerabilities
downstream.

<br>

## Where things stand

The repository that I work in contains somewhere above 500,000 lines of first
party Rust code. Around 99.7% of that is safe code. I did a rough categorization
of the remainder and it breaks down as follows:

- 958 unsafe blocks &mdash; FFI to C++
- 103 &mdash; FFI to OCaml
- 37 &mdash; FFI to Python
- 93 &mdash; would exist even if the whole codebase were Rust

From these numbers it's clear to me that a safe FFI story could substantially
assist in maintaining the long term health and correctness of this codebase as
we enter into the millions of lines. I have plans for this and will be writing
more about safe zero-overhead C++ FFI in 2020.

Note that other codebases may have a quite different ratio of unsafe code
depending on their priorities and requirements. For example [Libra] is a pure
Rust codebase and contains just 1 unsafe line per 165,000 lines of Rust, or
99.9994% safe code.

[Libra]: https://github.com/libra/libra

<br>

## Findings

Having examined around 3 dozen of the C++-related unsafe blocks and 2 dozen of
the pure Rust ones, so far I have discovered three soundness bugs. Two were in a
poorly designed library for interoperating with C++ string\_view and one was in
a poorly implemented library for per-thread counters. While researching this
article I also discovered [one soundness bug in Libra].

[one soundness bug in Libra]: https://github.com/libra/libra/pull/1949

This isn't great but it definitely does not call for panic. None of the four
bugs involves undefined behavior or memory unsafety actually present in the
project. They are soundness bugs affecting potential future misuses of a library
API, but in all cases no current uses were incorrect.

**The `unsafe` keyword made it possible to discover these bugs *before* they
became vulnerabilities.**

Without reading the vast majority of my codebase, I am able to have high
confidence that the hundreds of thousands of lines of code that depend only on
abstractions already reviewed by me are absent of memory safety and thread
safety bugs.

<br><br>

I hope that sharing this experience gives you an honest insight into Rust as a
safe language but a practical language at the same time. The impractical
alternatives, forbidding code that the language cannot know is safe, or treating
code that transitively relies on unsafe code as unsafe, do not make for a
language that is as safe and practical as Rust.
*/
#[macro_export]
macro_rules! _03__soundness_bugs {
    ({
        date:  "December 9, 2019",
        author:  "David Tolnay",
    }) => {};
}
