# xenopsd-ng

## Context

Following Xen community discussions around the futur of Xen toolstack, it was decided:

* to use a daemon like `xenopsd` project to build a toolstack on top of Xen
* this new toolstack should be decoupled and modular
* it would be integrated in the Xen project repo at some point
* it must be easy to contribute "vertically" (form Xen to the toolstack to expose and test a new feature), therefore OCaml is not acceptable from the community point of view (too hard).

Those requirements are making impossible the usage of `xenopsd` as-is. However, the concept itself is interesting, and people would like to discover more how it works, with a proof-of-concept showing how to achieve basic operations.

## Objectives

In short, the goal is to start reimplementing `xenopsd` in another language. "To start" because it's only an evaluation, and it might expose other problems on the lower level. This was a major concern: there's some cleaning and choices to make on Xen level itself (ed note: at `libxc` level I suppose?).

So this is a kind of "parallel" work between reimplementing and doing some work in Xen itself when we spot suboptimal cases.

## Languages

There's no community consencus yet on choosing the new `xenopsd-ng` language. This is why we'll try to implement PoC in different languages. This will probably help us to choose what's the best fit before going for a definitive decision.

Right now, the ideal language wouldn't require any big effort to integrate a new feature vertically (from Xen to the API), so it means the result should be:
* not vasly different from the lower stack
* simple enough so we don't lose contributors because of it (eg not OCaml)
* not too "risky" (memory safe would be fantastic)
* not too slow (does't it make sense for this? do we need to avoid a language with a GC?)
* not unfit for the end result (eg: Python is nice but at some scale it requires some discipline, C++ might be too complex)
* with tests as soon as possible
* capacity to be modular easily
* not difficult to potentially integrate "on the field" (do we need something statically linked?)

### Possible languages

For all of them: we need to evaluate if it fits for the role in the "global" aspect (writing an API that can be big)

#### Go

+ Relatively simple
+ Learning curve should be OK
- Got a GC

#### Rust

+ Memory safe!
+ No GC
- Learning curve? Hard to tell for our use case

#### C

+ Same language than Xen (homogeneity)
- Might be hard to have a complete "higher" level API written in such low level language? (maintenance, leaks…)

#### Python

+ Simple
- Might be badly used and that led to `xend`