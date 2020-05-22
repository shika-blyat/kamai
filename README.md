# KAMAI

<p align="center"><img src="assets/kamai_logo.png" alt="drawing" width="250"/></p>
Kamai (pronounced ˈkɑməï) is a small yet pragma.. no, I'm joking. Kamai is an experiment, probably going nowhere, that I'm making to learn stuff related to programming languages.
It's still in its extremely early stage, and even the goals/planned features aren't stable yet.

## Planned features

- [ ] A decent parser (WIP)
- [ ] Currying
- [ ] ADTs (Rust-like enums and tuple)
- [ ] Lifetime system (with non lexical lifetimes maybe ?)
- [ ] Multiple memory management strategy built in inside the langage (GC and unique owner RAII for the moment)
- [ ] Typeclasses/Traits and HKTs
- [ ] Mutability restriction, mainly based on Rust rules (but I'm not sure if whether or not Kamai rules will be exactly the same as Rust's)

Yes, Kamai is Rust with : currying, a built in GC and, probably, a locally space sensitive syntax. And I'll probably remove some things from this list and add new fancy features at some point, but for the moment that's how the list looks like. I'll try to update it as often as I think about it. 

## Goals

I'd love to bootstrap the interpreter/compiler, write some graphics application and maybe even a small kernel at some point. Oh and for the things that could reasonably happen in a near future, a working interpreter would already be really cool 

## Example

I don't have any example yet, because I'm not sure about the syntax yet, but I guess it would be something like that:

```haskell
import System::exit
import MetaInfo::(Line, Column)

data Option: T{
    Some T,
    None
}

unwrap (Some a) = a
unwrap _ = exit f"{Line}:{Column}\nUnwrapped on a Option::None value"
```

## Special thanks to
- [@mesabloo](https://github.com/Mesabloo) for the logo. Also check out [nihil](https://github.com/Mesabloo/nihil), his language, nihil's really cool :D. 