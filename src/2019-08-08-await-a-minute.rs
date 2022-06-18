/** # Await a minute, why bother?

<sup>*by [David Tolnay]&#8202;,&ensp;2019.08.08*</sup>

[David Tolnay]: https://github.com/dtolnay

<br>

Recently I have been retooling some core Rust libraries at $work to play nicely
with native async/await syntax. This note covers my thoughts on why this feature
is so important to our async codebase if it's "just" syntax sugar for a job that
could just be done using raw Futures instead.

- Comprehensible error handling;
- Native control flow;
- Borrowing.

<br>

## Comprehensible error handling

This boring thing has been the killer feature of await in my experience. I think
there is general understanding that await code can be easier to read and write
than `Future`-based code, but it hasn't been called out often enough just how
much of a difference this can make in Rust.

Developers who have worked with the futures 0.1 library in Rust are familiar
with using "combinators" on the `Future` trait to chain together sequential
stages of a computation, producing one `Future` at the end that will ultimately
be polled by a runtime for the computation to make progress through those
stages. This is a lot like working with combinators on the `Result` or `Option`
type. Methods like [`.map(...)`], [`.then(...)`], and [`.or_else(...)`] allow
acting on the success value or the error value of the computation so far, some
of them synchronously and other asynchronously.

[`.map(...)`]: https://docs.rs/futures/0.1.28/futures/future/trait.Future.html#method.map
[`.then(...)`]: https://docs.rs/futures/0.1.28/futures/future/trait.Future.html#method.then
[`.or_else(...)`]: https://docs.rs/futures/0.1.28/futures/future/trait.Future.html#method.or_else

Here is an example of `Future` combinators in action. This snippet is Real Code.
I have lightly simplified it to omit irrelevant details, but all but one of the
comments (the one starting with "snip") and all of the structure is exactly as
found in our codebase. This is glue code from a server that receives an incoming
serialized request, parses out the request arguments, hands them off to some
logic that determines what to respond back, and serializes that outgoing
response.

```
# use futures01::{future, Future, IntoFuture};
#
# struct Error;
# struct Res;
# type ProtocolEncodedFinal<P> = <<P as Protocol>::Deserializer as Deserializer>::EncodedFinal;
#
# trait Protocol: 'static {
#     type Deserializer: Deserializer;
# }
#
# trait Deserializer: Clone + Send {
#     type EncodedFinal;
# }
#
# enum MessageType {
#     Reply,
# }
#
# impl Res {
#     fn write<D>(&self, _de: &mut D) -> Result<D::EncodedFinal, Error>
#     where
#         D: Deserializer,
#     {
#         unimplemented!()
#     }
# }
#
# fn write_message<D, F>(
#     _protocol: D,
#     _method: &str,
#     _type: MessageType,
#     _write: F,
# ) -> Result<(), Error>
# where
#     D: Deserializer,
#     F: FnOnce(&mut D) -> Result<D::EncodedFinal, Error>,
# {
#     unimplemented!()
# }
#
# struct Example<P> {
#     service: Service,
#     protocol: P,
# }
#
# struct Service;
#
# impl Service {
#     fn get_counters(&self, args: ()) -> impl Future<Item = Res, Error = Error> {
#         future::ok(Res)
#     }
# }
#
# impl<P> Example<P>
# where
#     P: Protocol,
#     P::Deserializer: Deserializer<EncodedFinal = ()>,
# {
fn handle_get_counters(
    &self,
    p: &mut P::Deserializer,
) -> impl Future<Item = ProtocolEncodedFinal<P>, Error = Error> + Send + 'static {
    // Wrap arg decoding and the svc call in a closure so we can use `?` and
    // capture the error
    let ret: Result<_, Error> = (|| {
        let args = {/* snip: some code using `?` */};
        Ok(self.service.get_counters(args))
    })(); // Result<Future<Res, Exn>, E>

    // Work out how to handle the future from the method. This is wrapped inside
    // a Result which we chain along, so that we can ultimately return a single
    // Future type.
    let ret = ret.map(|res| { // Result<Future<Res, Exn>, E>
        # let p = p.clone();
        // res: Future<Res, Exn>
        res.then(move |res| {
            res.and_then(move |res| write_message(
                p, "getCounters", MessageType::Reply, |p| res.write(p),
            ))
        })
    }); // Result<Future<Bytes, E>, E>
    ret.into_future().flatten()
}
# }
```

At a high level this function is doing something extremely basic: do some
fallible synchronous work, then some fallible asynchronous work, then some
fallible synchronous work. From the comments (good job!) and the complexity of
the implementation, it's clear that this code wasn't the no-brainer that it
should be. It likely took a skilled Rust developer *at least* 10 minutes to get
something like this past the type checker, including some time in the `futures`
docs. For someone only partway through the Rust book for the first time, code
like this is basically impossible to write or extend.

Here is the same code after introducing async/await in the server library. The
structure pops out immediately. There is some fallible synchronous work, then
the fallible asynchronous call, and some fallible synchronous work at the end.

```
# struct Error;
# struct Res;
# type ProtocolEncodedFinal<P> = <<P as Protocol>::Deserializer as Deserializer>::EncodedFinal;
#
# trait Protocol {
#     type Deserializer: Deserializer;
# }
#
# trait Deserializer {
#     type EncodedFinal;
# }
#
# enum MessageType {
#     Reply,
# }
#
# impl Res {
#     fn write<D>(&self, _de: &mut D) -> Result<D::EncodedFinal, Error>
#     where
#         D: Deserializer,
#     {
#         unimplemented!()
#     }
# }
#
# fn write_message<D, F>(
#     _protocol: &mut D,
#     _method: &str,
#     _type: MessageType,
#     _write: F,
# ) -> Result<(), Error>
# where
#     D: Deserializer,
#     F: FnOnce(&mut D) -> Result<D::EncodedFinal, Error>,
# {
#     unimplemented!()
# }
#
# struct Example<P> {
#     service: Service,
#     protocol: P,
# }
#
# struct Service;
#
# impl Service {
#     async fn get_counters(&self, args: ()) -> Result<Res, Error> {
#         unimplemented!()
#     }
# }
#
# impl<P> Example<P>
# where
#     P: Protocol,
#     P::Deserializer: Deserializer<EncodedFinal = ()>,
# {
async fn handle_get_counters(
    &self,
    p: &mut P::Deserializer,
) -> Result<ProtocolEncodedFinal<P>, Error> {
    let args = {/* snip: some code using `?` */};
    let res = self.service.get_counters(args).await?;
    let enc = write_message(p, "getCounters", MessageType::Reply, |p| res.write(p))?;
    Ok(enc)
}
# }
```

Rather than tetrising together a bunch of `map` and `and_then` and `flatten`
combinators with [ridiculous signatures], practically the only thing to know is
that we write `.await` after asynchronous things and `?` after fallible things.
This is code that a beginner could write and a beginner could maintain, but it's
a big relief at any level of experience.

[ridiculous signatures]: https://docs.rs/futures/0.1.28/futures/future/trait.Future.html#method.flatten

The error handling complexity of futures appears everywhere. Here is another
Real Code snippet, before and after introducing await.

```
# use futures01::{future, Future};
#
# struct ServiceFramework;
# struct BuildModule;
# struct ThriftStatsModule;
# struct ProfileModule;
#
# impl ServiceFramework {
#     fn new(name: &str, thrift: (), port: ()) -> Result<Self, ()> {
#         Ok(ServiceFramework)
#     }
#
#     fn add_module<M>(&mut self, module: M) -> Result<(), ()> {
#         Ok(())
#     }
#
#     fn serve(&mut self) -> impl Future<Error = ()> {
#         future::ok(())
#     }
# }
#
# let thrift = ();
# let port = ();
#
let mut svc = ServiceFramework::new("email_validator_service", thrift, port).unwrap();
let add_modules = svc
    .add_module(BuildModule)
    .and_then(|_| svc.add_module(ThriftStatsModule))
    .and_then(|_| svc.add_module(ProfileModule));
future::result(add_modules).and_then(|_| svc.serve())
# ;
```

```
# struct ServiceFramework;
# struct BuildModule;
# struct ThriftStatsModule;
# struct ProfileModule;
#
# impl ServiceFramework {
#     fn new(name: &str, thrift: (), port: ()) -> Result<Self, ()> {
#         unimplemented!()
#     }
#
#     fn add_module<M>(&mut self, module: M) -> Result<(), ()> {
#         unimplemented!()
#     }
#
#     async fn serve(&mut self) -> Result<(), ()> {
#         unimplemented!()
#     }
# }
#
# async fn try_main() -> Result<(), ()> {
# let thrift = ();
# let port = ();
#
let mut svc = ServiceFramework::new("email_validator_service", thrift, port)?;
svc.add_module(BuildModule)?;
svc.add_module(ThriftStatsModule)?;
svc.add_module(ProfileModule)?;
svc.serve().await?;
#     Ok(())
# }
```

Ask yourself: if I wanted to insert a fallible call (maybe synchronous, maybe
asynchronous) between some pair of those existing calls, how long would it take
me to figure out the right combinator in the top code? How long would it take in
the bottom code?

<br>

## Native control flow

The error handling simplifications in the previous point largely arise from the
ability to replace library-based control flow (combinators) with the `?`
operator to propagate errors. `?` works equally well in async functions as it
always has in ordinary synchronous functions.

But `?` is just one example of syntax-based native control flow. Like most
languages, Rust has some other control flow syntax, such as the `if` keyword for
branching and the `while` keyword for looping.

Combinators were manageable for the most common patterns of
do-this-then-that-then-that, but as soon as someone needs control flow slightly
outside of what the predefined combinators support, it quickly gets very
complicated. Consider some asynchronous call that we want to repeat while some
asynchronous function returns true, the equivalent of this trivial await
snippet:

```
# async fn keep_going() -> bool {
#     unimplemented!()
# }
#
# async fn do_the_thing() -> Result<(), ()> {
#     unimplemented!()
# }
#
# async fn try_main() -> Result<(), ()> {
while keep_going().await {
    do_the_thing().await?;
}
#     Ok(())
# }
```

Even something this basic would be a challenge to express with `Future`
combinators for an expert Rust programmer, and for a beginner it would be
practically impossible.

Sometimes combinators don't get the job done (or they would, but the developer
isn't familiar enough with the library to find which set of combinators to
string together for the behavior they need) and we fall back to handwritten
futures. Here is some Real Code that is one part of a 195-line state machine
that could be replaced by a far clearer 12 line async fn with identical behavior
and performance.

```
# use futures01::{Async, Future, Poll};
#
# enum EncodeState<W> {
#     Start(StartFut),
#     Part((), ()),
#     EndOfStream((), ()),
#     Finish(W),
#     Done,
#     Invalid,
# }
#
# struct CompressedRead<W>(W);
#
# struct Error;
#
# struct StartFut;
#
# impl StartFut {
#     fn finish(self) -> ((), ()) {
#         unimplemented!()
#     }
# }
#
# impl Future for StartFut {
#     type Item = ();
#     type Error = Error;
#     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
#         unimplemented!()
#     }
# }
#
# struct Example<W>(W);
#
# impl<W> Example<W> {
#     fn poll_next_part(iter: (), sink: ()) -> (Poll<W, Error>, EncodeState<W>) {
#         unimplemented!()
#     }
#
#     fn poll_part(part_fut: (), iter: ()) -> (Poll<W, Error>, EncodeState<W>) {
#         unimplemented!()
#     }
#
#     fn poll_eos(sink: (), eos_written: ()) -> (Poll<W, Error>, EncodeState<W>) {
#         unimplemented!()
#     }
#
#     fn poll_finish(_: CompressedRead<W>) -> (Poll<W, Error>, EncodeState<W>) {
#         unimplemented!()
#     }
#
fn poll_next(state: EncodeState<W>) -> (Poll<W, Error>, EncodeState<W>) {
    match state {
        EncodeState::Start(mut start_state) => {
            match start_state.poll() {
                Ok(Async::Ready(())) => {
                    // Writing to the stream header is done. Set up the sink
                    // and remaining parts.
                    let (iter, sink) = start_state.finish();
                    Self::poll_next_part(iter, sink)
                }
                Ok(Async::NotReady) => (Ok(Async::NotReady), EncodeState::Start(start_state)),
                Err(err) => {
                    // Somehow writing out the stream header failed. Not
                    // much to do here unfortunately -- we must abort the future.
                    (Err(err), EncodeState::Invalid)
                }
            }
        }
        EncodeState::Part(part_fut, iter) => Self::poll_part(part_fut, iter),
        EncodeState::EndOfStream(sink, eos_written) => Self::poll_eos(sink, eos_written),
        EncodeState::Finish(compressor) => Self::poll_finish(CompressedRead(compressor)),
        EncodeState::Done => panic!("polled future after it is complete"),
        EncodeState::Invalid => panic!("polled future after it returned an error"),
    }
}
# }
```

In contrast to library-based control flow and handwritten futures, the
async/await language feature makes everything that a beginner would read about
control flow in the Rust book directly applicable to operating in an
asynchronous codebase.

<br>

## Borrowing

Aaron Turon covered this in an article called *[Borrowing in async code]* from
last year, even before the async/await feature was fully designed (so the code
snippets may look odd). I'll quote and then summarize, but check out the link
for Aaron's full explanation.

[Borrowing in async code]: https://aturon.github.io/tech/2018/04/24/async-borrowing/

> *The bottom line is that async/await isn't just about not having to use
> combinators like `and_then`. It also fundamentally changes API design in the
> async world, allowing us to use borrowing in the idiomatic style. Those who
> have written much futures-based code in Rust will be able to tell you just how
> big a deal this is.*

Almost all existing `Future`-based code in our codebase is written using
`'static` futures. You can see the `'static` bound in the first
`handle_get_counters` snippet at the top of this page. That means futures are
constrained not to refer to any data outside of what is owned by the future
itself. These futures are ultimately tossed onto executors like thread pools,
and might run there beyond the lifetime of any particular stack frame except the
`'static` lifetime.

To build `'static` futures we make heavy use of cloning and `Arc<Mutex<T>>`.
This makes everything look like owned values so the borrow checker doesn't come
into play, but also we miss out on the benefits of the borrow checker for
writing safe readable code.

Similar to how `Mutex` is a safe Sync-maker (wrapping something that is not
[Sync] to expose it safely as Sync), `async` blocks can play the role of a safe
`'static`-maker. The async block can `await` a future that operates on borrowed
data, while still being `'static` overall and thus spawnable on a thread pool or
other executor. Aaron walks through an example of this involving asynchronously
filling a buffer &mdash; check it out.

[Sync]: https://doc.rust-lang.org/std/marker/trait.Sync.html

<br>

## [https://areweasyncyet.rs/]

Async/await syntax is only available in the nightly compiler for now, but is on
track to stabilize in Rust 1.38 next month. You can following along with news
about the async ecosystem and stabilization process at
[https://areweasyncyet.rs/].

[https://areweasyncyet.rs/]: https://areweasyncyet.rs/
*/
#[macro_export]
macro_rules! _01__await_a_minute {
    ({
        date:  "August 8, 2019",
        author:  "David Tolnay",
    }) => {};
}
