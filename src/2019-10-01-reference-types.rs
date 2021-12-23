/** # Accurate mental model for Rust's reference types

<sup>*by [David Tolnay]&#8202;,&ensp;2019.10.01*</sup>

[David Tolnay]: https://github.com/dtolnay

<br>

Rust's [ownership and borrowing system][ownership] involves the use of
*references* to operate on borrowed data, and the type system distinguishes two
different fundamental reference types. In code they are spelled **`&T`** and
 **`&mut T`**.

`&mut T` is commonly known as a "mutable reference" to data of type `T`. By
juxtaposition, `&T` is then an "immutable reference" or "const reference" to
`T`. These names are fine and reasonably intuitive for Rust beginners, but this
article lays out the motivation for preferring the names "shared reference" and
"exclusive reference" as you grow beyond the beginner stage and get into library
design and some more advanced aspects of the language.

[ownership]: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html

<br>

## The beginner's understanding

As described in the [References and Borrowing][borrowing] chapter of the Rust
Book, a function that takes an argument by immutable reference is allowed to
read the data behind the reference:

[borrowing]: https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html

```
struct Point {
    x: u32,
    y: u32,
}

fn print_point(pt: &Point) {
    println!("x={} y={}", pt.x, pt.y);
}
```

but is not allowed to mutate that data:

```compile_fail
# struct Point {
#     x: u32,
#     y: u32,
# }
#
fn embiggen_x(pt: &Point) {
    pt.x = pt.x * 2;
}
```

```console
error[E0594]: cannot assign to `pt.x` which is behind a `&` reference
 --> src/main.rs
  |
1 | fn embiggen_x(pt: &Point) {
  |                   ------ help: consider changing this to be a mutable reference: `&mut Point`
2 |     pt.x = pt.x * 2;
  |     ^^^^^^^^^^^^^^^ `pt` is a `&` reference, so the data it refers to cannot be written
```

In order to mutate fields of a struct, or call mutating methods such as
appending to a vector, the argument must be taken by `&mut` reference.

```
# struct Point {
#     x: u32,
#     y: u32,
# }
#
fn embiggen_x(pt: &mut Point) {
    pt.x = pt.x * 2; // okay
}
```

This distinction, and the terminology of "immutable reference" and "mutable
reference", is typically adequate for writing one's first few toy programs with
Rust.

<br>

## It falls apart

Sooner or later you will encounter a library signature that flatly contradicts
the beginner's mental model of Rust references. Let's take a look at the `store`
method of [`AtomicU32`] from the standard library as one example of this. The
signature is:

[`AtomicU32`]: https://doc.rust-lang.org/std/sync/atomic/struct.AtomicU32.html

```
# struct AtomicU32;
#
impl AtomicU32 {
    # const IGNORE: &'static str = stringify! {
    pub fn store(&self, val: u32, order: Ordering);
    # };
}
```

You give it a u32 value, and it atomically changes the number inside the
`AtomicU32` to hold the value you gave. We might call the `store` method as:

```
# use std::sync::atomic::{AtomicU32, Ordering};
#
static COUNTER: AtomicU32 = AtomicU32::new(0);

fn reset() {
    COUNTER.store(0, Ordering::Release);
}
```

The `Ordering` parameter can be ignored for the purpose of this discussion; it
has to do with the [C11 memory model for atomic operations][atomics].

[atomics]: https://doc.rust-lang.org/nomicon/atomics.html

But the fact that `AtomicU32::store` takes self by immutable reference **should
feel deeply uncomfortable** under the beginner's mental model. Sure the mutation
is done atomically, but how can it be correct that we mutate something under an
immutable reference? Is this a typo in the standard library? If intentional, it
certainly feels hacky, or even dangerous. How is this method safe? How is it not
undefined behavior?

For former C++ programmers it calls to mind certain abuses of `const_cast` in
C++, where maybe the author was never really sure whether they were violating
some esoteric language law that would break the behavior of the code later on,
even if it currently appears to work.

Certainly in C++ the atomic mutation methods like [`std::atomic<T>::store`] all
act on mutable references only. Storing through a const reference to a C++
atomic won't compile, as one should expect.

[`std::atomic<T>::store`]: https://en.cppreference.com/w/cpp/atomic/atomic/store

```cpp
// C++

#include <atomic>

void test(const std::atomic<unsigned>& val) {
  val.store(0);
}
```

```console
test.cc:4:7: error: no matching member function for call to 'store'
  val.store(0);
  ~~~~^~~~~
/usr/include/c++/5.4.0/bits/atomic_base.h:367:7: note: candidate function not viable: no known conversion from 'const std::atomic<unsigned int>' to 'std::__atomic_base<unsigned int>' for object argument
      store(__int_type __i, memory_order __m = memory_order_seq_cst) noexcept
      ^
/usr/include/c++/5.4.0/bits/atomic_base.h:378:7: note: candidate function not viable: no known conversion from 'const std::atomic<unsigned int>' to 'volatile std::__atomic_base<unsigned int>' for object argument
      store(__int_type __i,
      ^
```

Something is wrong. It turns out to be the beginner's understanding of what the
Rust `&` and `&mut` reference types mean.

<br>

## Better names

`&T` is not an "immutable reference" or "const reference" to data of type `T`
&mdash; it is a "shared reference". And `&mut T` is not a "mutable reference"
&mdash; it is an "exclusive reference".

An exclusive reference means that no other reference to the same value could
possibly exist at the same time. A shared reference means that other references
to the same value *might* exist, possibly on other threads (if `T` implements
`Sync`) or the caller's stack frame on the current thread. Guaranteeing that
exclusive references really are exclusive is one of the key roles of the Rust
borrow checker.

Let's stare at the signature of `AtomicU32::store` again.

```
# struct AtomicU32;
#
impl AtomicU32 {
    # const IGNORE: &'static str = stringify! {
    pub fn store(&self, val: u32, order: Ordering);
    # };
}
```

This time **it should feel totally natural** that this function takes the atomic
u32 by shared reference. *Of course* this function is fine with other references
to the same `AtomicU32` existing at the same time. *The whole point* of atomics
is allowing concurrent loads and stores without inducing a data race. If the
library refused to allow other references to exist during the call to `store`,
there would hardly be a point to doing it atomically.

The reason exclusive references always behave as mutable is because if no other
code is looking at the same data, we won't cause a data race by mutating it
care-free. A data race is when data is operated on from two or more places at
the same time and at least one is mutating, producing unspecifiable results or
memory unsafety. But via atomics or other forms of interior mutability discussed
below, mutating through a shared reference can be safe too.

Fully internalizing the terminology "shared reference" and "exclusive
reference", learning to think in terms of them, is an important milestone in
learning to make the most of Rust and its tremendous safety guarantees.

<br>

## Pedagogy

I don't think it is bad for `&` and `&mut` to be introduced at first as
immutable vs mutable references. The learning curve is difficult enough without
frontloading the content of this article. As far as a beginner would be
concerned, ability to mutate will be the most significant practical difference
between the two reference types.

What I would like to accomplish with this page is to establish that shifting
from the "immutable reference"/"mutable reference" mental model to the "shared
reference"/"exclusive reference" mental model is a necessary step that learners
should be encouraged to take at the right time, and for this page to help them
take it. A good time to link someone to this page is when they are first
surprised or confused by some library function taking `&` when they would expect
it to require `&mut`.

After someone has internalized references as being about shared vs exclusive
access, I think it is fine to continue saying "mutable reference" as a
convenience since the keyword is `mut` after all; just keep in mind that data
behind a shared reference *may also* be mutable sometimes. On the other hand for
shared references I would recommend to always think and say "shared reference"
rather than "immutable reference" or "const reference".

<br>

## Addendum: interior mutability

The term for safe APIs that support mutation through a shared reference in Rust
is "interior mutability".

I used `AtomicU32` as an example above because I find that it evokes the most
striking rift between deeply-uncomfortable and totally-natural as you shift from
the beginner's mental model to the correct one. While atomics are an important
building block for multithreaded code, interior mutability is equally relevant
on a single thread as well.

The standard library type [`UnsafeCell<T>`] is *the only* way to hold data that
is mutable through a shared reference. This is an unsafe low-level building
block that we would almost never use directly. All other interior mutability is
built as safe abstractions around an `UnsafeCell`, with a variety of
properties and requirements as appropriate to different use cases.
(Fundamentally Rust is a language for building safe abstractions, and this is
one of the areas where that is most apparent.)

[`UnsafeCell<T>`]: https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html

Beyond atomics, other safe abstractions in the standard library built on
interior mutability include:

- [`Cell<T>`] &mdash; we can perform mutation even when other references to the
  same `Cell<T>` may exist, and it's safe because the API enforces:

    - it's impossible for more than one thread to hold references to the same
      `Cell<T>` at a time because `Cell<T>` does not implement the `Sync` trait,
      i.e. `Cell<T>` is single threaded;

    - and it's impossible to obtain a reference to the contents within the
      `Cell<T>`, as such references could be invalidated by a mutation; instead
      all access is done by copying data out of the cell.

- [`RefCell<T>`] &mdash; we can perform mutation even when other references to
  the same `RefCell<T>` may exist, and it's safe because the API enforces:

    - `RefCell<T>` is single threaded so it's impossible for multiple threads
      to refer to the same one, similar to `Cell<T>`;

    - and within the one thread, dynamically checked borrow rules will detect
      and prevent attempts to mutate while a reader is holding a reference into
      the content of the `RefCell`.

- [`Mutex<T>`] &mdash; we can perform mutation even when other references to the
  same `Mutex<T>` may exist, and it's safe because the API enforces:

    - only one of the references may operate on the inner `T` at a time, whether
      reading or writing; other accesses will block until the current one has
      released its lock.

- [`RwLock<T>`] &mdash; we can perform mutation even when other references to
  the same `RwLock<T>` may exist, and it's safe because the API enforces:

    - only one of the references may be used to mutate the `T` at a time, and
      only while no other references are being used for reading; accesses will
      block to meet these requirements.

[`Cell<T>`]: https://doc.rust-lang.org/std/cell/struct.Cell.html
[`RefCell<T>`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
[`Mutex<T>`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
[`RwLock<T>`]: https://doc.rust-lang.org/std/sync/struct.RwLock.html
*/
#[macro_export]
macro_rules! _02__reference_types {
    ({
        date:  "October 1, 2019",
        author:  "David Tolnay",
    }) => {};
}
