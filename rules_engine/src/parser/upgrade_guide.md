We are working on migrating to a new version of the Chumsky crate, which is a
from-scratch rewrite of the original code. Here is a migration guide from the
author of Chumsky:

## Guide

I think a fully comprehensive migration guide would be infeasible and would likely be actively unhelpful: there have been so many small changes that documenting all of them would produce a thoroughly obtuse document. There are a few main ones though:

The Parser trait has now gained a lifetime parameter, representing the lifetime of the input. Most of the time you just want to have your parser function be generic over it, like fn my_parser<'src>() -> impl Parser<'src, I, O> { ... }.

The I parameter of Parser<'src, I, O> now refers to the entire input, not just the token type. For example, if your parser previously parsed Tokens, your input type will likely be &'src [Token]. If your parser previously parsed characters, the input type will be &'src str.

The optional E parameter of Parser<'src, I, O, E>, which previously indicated the error type, has been renamed as the 'Extra' parameter, which continues to specify the error type, along with other features like state, context, etc. There is a convenient short-hand in the form of the the extra::Err<E> type alias, which preserves the same behaviour. Here is an example.

Parsers are no longer lazy by default. Previously, one had to add .then_ignore(end()) to the top-level parser to 'force' it to keep consuming tokens until the end of the input. This is no longer the case, calling my_parser.parse(...) will produce an error if all input is not consumed (this is almost always the behaviour you want anyway). If the old behaviour is still desired for some reason, there is a lazy combinator.

Many features of Parser (such as foldr, as you mention) have been split off into the new IterParser trait, which represents parsers that produce multiple values. For example, doing x.repeated() now gives you an IterParser, and you can call collect on it to collect the output elements into a collection like x.repeated().collect::<Vec<_>>(), turning it back into a Parser.

There's a new combinator, pratt, which makes writing expression parsers much simpler. If you have a complex expression parser that feels unwieldy, you might want to take a look at it. Precedence climbing still works just as it did before though.

The behaviour of parsers has changed subtly. Now, parsers that have their output go unused will not even attempt to generate output in the first place (thanks to the new internal optimiser, which accounts for the lion's share of the 0.10 performance improvements). For example, the closure in a.map(|_| ...).ignore_then(b) will never even get called. This should only matter if your mapping closures are impure, which I suspect is not the case for 95% of users. More information is available in the guide.

The chain combinator has been removed. Its implementation was overly expensive. It is recommended to use the new 'to_slice' combinator, which allow you to fetch a slice of the output covered by a parser. For example,

none_of('"')
    .repeated()
    .to_slice()
    .delimited_by(just('"'), just('"'))
    .parse("\"hello, world!\"")
will produce hello, world as output.

The map_with_span combinator has been replaced with the new map_with combinator, which serves the same role, but also allows access to parser state, slices, and parser context.

take_until was removed because its use was a ambiguous (it wasn't clear to many users whether it would or wouldn't consume the terminating pattern). It's been replaced by several simpler combinators that can be used in combination (hence the name!) to produce the same or similar effects. They are:

any(), which already existed
a.and_is(b), which parses an a if b would also be matched in the same place
a.not(), which parses nothing successfully (like empty), but only if a is not matched
You can combine them to emulate take_until(a) like any().and_is(a.not()).repeated().then(a): it's a bit more wordy, but it affords much more control over exactly how parsing happens - and you can always lift it into a closure if you want to do less typing.

error::Simple has been renamed to error::Rich in chumsky 0.10.0, with the name Simple being reused for an actually simple error type. Rich::custom still exists and has an almost identical type signature.
I think this covers the main points of friction you're likely to encounter. With
the exception of pratt, I've avoided talking about new features to keep this
simple. 95% of things that worked with chumsky 0.9 will continue to work with
chumsky 0.10, if tweaked appropriately.

## Using Chumsky

# Creating Parsers

Creating parsers
Because chumsky uses typed combinators to express parsers, parser type signatures can become a little unwieldy. For this reason, it’s common practice to leave the heavy work of dealing with types to the compiler by making use of Rust’s impl Trait syntax.

Here’s an example of a typical parser function. We’ll go over what each part means.

//        (1)            (2)              (3)    (4)
//        _|__       _____|_____       ____|____  |_
fn parser<'src>() -> impl Parser<'src, &'src str, ()> {
    end() // --(5)
}
Parsers are parameterised over the lifetime of their inputs. Because we don’t yet know what input our parser will be used to parse, we declare a generic lifetime, 'src, to allow the parser to work with whatever input lifetime it needs to work with.

Because large parsers can have rather unwieldy types, we save ourselves the need to declare the exact return type with Rust’s impl Trait syntax. This says to the compiler “we don’t actually care what type is returned here, but it needs to implement the Parser<'src, &'src, str, ()> trait, you figure it out”. Note that, unlike dyn Trait syntax, impl Trait has no runtime cost: the compiler simply hides the type from you rather than performing type erasure, which would require performing dynamic dispatch while your code is running.

The first type parameter (i.e: ignoring the lifetime parameter) of the Parser trait is the input type. Inputs must implement the Input trait. Examples of inputs include strings, slices, arrays, Streams, and much more. For now we specify that this parser can only operate upon string slices: but it is also possible to introduce the input type as a generic type parameter like I: Input<'src> instead if you want your parser to be generic across more than just string slices.

The second type parameter of the Parser trait is the output type. This is the type of the value that your parser will eventually give you, assuming that parsing was successful. For now, we just use an output type of [()], i.e: nothing.

Because this is just an example parser, the implementation is just a single parser primitive, end. This is a primitive that recognises only the end of the input and generates an error if it does not find it. This means that our parser effectively just checks that we pass it an empty string: anything else will generate an error.

Note that this function only creates the parser: it does not, by itself, perform
any parsing.

# Using Parsers

It’s all very well creating parsers but in order to write useful programs, we need to invoke them. Chumsky provides several functions for this, but the main two are:

Parser::parse: parses an input, generating an output value and/or any errors that were encountered along the way

Parser::check: checks that an input is valid, generating any errors that were encountered along the way

Both functions give us back a ParseResult. You can think of this sort of like Rust’s regular Result type, except it allows both outputs and errors to be generated at the same time (although we won’t yet use this functionality). If you just want parsing to be an all-or-nothing affair, you can use ParseResult::into_result to convert this into a regular Result.

Let’s write some tests for the parser we wrote in the last section.

#[test]
fn test_parser() {
    // Our parser expects empty strings, so this should parse successfully
    assert_eq!(parser().parse("").into_result(), Ok(()));

    // Anything other than an empty string should produce an error
    assert!(parser().parse("123").has_errors());
}
Hopefully, this code is fairly self-explanatory. We call parse() (the function we wrote in the previous section) to create an instance of our parsers, and then we call Parser::parse on it with the desired input to actually do some parsing. The return value is the result of the parse.

From here, the world is your lobster: you can move on to the tutorial sections
of this guide or you can jump write into writing parsers. The main repository
has plenty of examples to use as a reference and the crate has documentation
that will help guide you, with many examples.

## Advice on Compiler Errors

Chumsky is a combinator crate and leans heavily into Rust’s type system (traits, generics, etc.) in order to combine high performance and ergonomics. Unfortunately, the Rust compiler can still struggle to generate useful error messages for large chumsky parsers (although things have improved substantially in recent releases!). When you hit a compiler error you’re struggling to understand, you should:

Always solve the first error that Rust generates. Rust generates errors in the order that it finds them, so the first error is usually reliably accurate while later errors tend to get increasingly speculative as the compiler needs to make more and more assumptions about your program to handle prior errors. This often results in many additional ‘phantom errors’: errors that muddy the water and make it look like the problem is more complicated to solve than it actually is.

Reduce the size of types. Thankfully Rust has recently taken steps to avoid printing extremely long type signatures out to the terminal. Even so, parser types can still be rather large. You can reduce this problem by commenting out unnecessary parts of your parser, or using .simplify() on parsers that contribute to the error to simplify their types.

Complaints about types ‘not implementing Parser’ are more often than not a
failure to fulfil the obligations that come with implementing the trait. For
example, recursive() requires that the inner parser implements Clone: a parser
that doesn’t (because, say, you moved a non-cloneable type into the closure)
can’t be used with recursive() and so Rust will translate this, in its parlance,
to the type not implementing Parser.

# Full Documentation

What are parser combinators?
Chumsky is a declarative parser combinator library. Let’s break that down to explain what it means.

Parsers
Parsers are programs (or, for our purposes, functions) which take unstructured inputs and produce structured outputs according to a set of rules called a grammar.

What counts as structured and unstructured depends on the context. To a lexer, a list of tokens might count as a structured output, but to the parser that consumes them as an input, they look rather less structured.

Because the set of possible unstructured inputs to a parser (such as bytes in a text file) is generally larger than those that can be correctly translated to the structured output according to the grammar rules (such as an Abstract Syntax Tree), parsers need a way to generate errors when these invalid inputs are encountered.

Declarative style
If you’ve hand-written a parser before, it was likely in the imperative style: which is to say that you used code to tell your program how to parse inputs. This is a valid approach to writing parsers, and many successful parsers are written in an imperative style.

However, imperative-style parsers are often extremely ‘noisy’: resulting in parser code that is long, difficult to maintain, is hard to read, time-consuming to optimise, and easy to break, and difficult to debug.

In comparison, chumsky encourages you to write declarative parsers. In the declarative style, instead of telling your code how to parse inputs, you tell it what to parse. This is a much more grounded and to-the-point approach to implementing parsers, allowing you to focus on the grammar rules you want to parse instead of spending ages debugging and maintaining imperative-style parser logic.

If you search for information about declarative parsers (and in particular, parser combinators), you’ll often hear it said that they’re slow and imprecise. While this might have been true in decades gone by, modern optimising compilers - and in particular Rust’s powerful type system - make the development of expressive declarative parsers that are as fast (or faster!) than hand-written parsers both easy and quick.

Combinators
Modern software is written primarily through through the use of functions. Each function performs a specific task and may call out to sub-functions. To create a whole program, it is necessary to combine functions to get the desired behaviour of the program as a whole.

Parser combinators take this approach and apply it to parsing: a parser written with a combinator approach is composed of many smaller sub-parsers that are each able to process a sub-section of the overall grammar rules. These sub-parsers are then combined with parser operators known as combinators that define how they relate to one-another.

Chumsky comes with many combinators that allow the creation of even very complex grammars. Indeed, parsers for entire programming languages may be easily written with chumsky.

As with most things, it’s turtles all the way down: each sub-parser is then composed of sub-sub-parsers, which is itself composed of sub-sub-sub-parsers, until we reach the most basic elements of the parser logic.

🐢

Primitives
Primitives are the most basic elements of chumsky’s parser logic. They are built-in components provided by chumsky (although it is possible to write your own!). Primitives each perform a very simple action that by itself seems almost trivial. For example, they might recognise a specific keyword or even just a single character.

Chumsky comes with several primitive parsers that each perform a specific job.

API features
The Parser trait
A fundamental concept in chumsky is that of the Parser trait. All parser (both combinators and primitives) implement it and the combinator methods on it are the primary way through which a parser is defined.

Parser also provides several invocation methods such as Parser::parse and Parser::check: these functions allow you to actually give inputs to your parser and have it generate outputs and/or errors.

Check out the primitive, combinator, recursive, and regex modules for examples of some of the parsers that chumsky provides.

The Input trait
The Input trait is implemented by all types that can act as inputs to chumsky parsers. For example, it is implemented by types such as:

&[T]: Array slices

&str: String slices

Stream<I>: Dynamically-growing token streams

Certain inputs have special properties. For example, it is possible to borrow &T tokens from &[T] array slices, but not chars from &str string slices (due to their UTF-8 encoding). Additionally, some inputs can have sub-slices taken from them. All of these operations are potentially useful to a parser, so chumsky expresses them with a set of extension traits that add extra functionality on top of the base Input trait:

ValueInput: for inputs that can have tokens copied/cloned from them by-value

BorrowInput: for inputs that can have individual tokens borrowed from them

SliceInput: for inputs that can have entire sub-slices of tokens borrowed from them

StrInput: for inputs that ‘look like’ text strings: ASCII byte slices (&[u8]) and UTF-8 string slices (&str)

Taken together, these traits give chumsky the power to use many different types as input: bytes, strings, tokens, token trees, iterators, and much more besides.

The Error trait
As discussed previously, parsers commonly need to be able to handle inputs that don’t conform to the grammar rules that they implement. To do this, they need to be able to emit errors that can then be processed by either the system that invoked the parser, or by a human user, in order to communicate what went wrong.

Chumsky provides support for expressive error generation through its Error trait, along with a series of built-in error types that have different tradeoffs:

EmptyErr: the default ‘null’ error that doesn’t record any useful information other than the fact that an error occurred

Cheap: a very efficient error type that records only the span of the input that triggered the error

Simple: a simplistic error type that records both the span that triggered the error and whatever token was erroneously found

Rich: a very information-rich error type that records:

The span that triggered the error

The token that was erroneously found instead

A list of tokens or patterns that were expected at the span location instead

Rich also supports many additional features such as custom error messages, labelling (see Parser::labelled) and error merging.

Obviously, errors that express more detailed information are also slower to generate and hence reduce the performance of the overall parser. In benchmarks, we tend to find that parsers using Rich typically run at about half the speed as those using EmptyErr, although this is very likely to improve as time goes on.

It is typical to take the data encoded in these types and give them to a ‘diagnostic generator’, a tool intended to turn error information into pretty human-readable displays suited for printing into a terminal, displaying in an IDE, or whatever other form of output is required.

The Span trait
Spans are ranges (usually byte offsets, but you can use whatever is most convenient for you) in the original source code that can be used to reference sections of the code in error or warning messages.

Chumsky has full support for spans and also allows you to define your own custom spans with ease by simply implementing the Span trait. Additionally, chumsky comes with a built-in span type, SimpleSpan, and a variety of implementations for types in Rust’s standard library such as std::ops::Range<usize>.

Chumsky will use its internal knowledge of your parser to generate spans for you
whenever you need them, such as for attaching to nodes of an abstract syntax
tree. See Parser::map_with for more information.

# Meet the Parsers

Meet The Parsers
Chumsky provides a whole suite of parser components, both primitives and combinators. This page lists the ones you’ll most commonly use. They’re roughly ranked in order of importance, with the most commonly used at the top of each list.

As a reminder: primitives are the most basic building blocks of a parser, while combinators allow you to combine them together into parsers that can handle increasingly complex grammars.

Note that when the term ‘recognises’ is used, it means that all other inputs are rejected by the parser, resulting in backtracing or an error.

Each parser has inline documentation, including longer and more useful examples.

Primitives

Combinators

Combining parsers

Generating and manipulating outputs

Handling and emitting errors

Text-oriented parsing

Utility and error recovery

Backtracking and input manipulation

Context-sensitive parsing

Primitives
Primitives are the most basic building blocks of a parser and typically perform a very simple action such as recognising a particular token or set of tokens.

Name	Examples	Description
just	just('a'), just("hello")	Recognises a specific token or an exact ordered sequence of tokens (see Seq and OrderedSeq).
none_of	none_of(';'), none_of("xyz")	Recognises any single token that is not part of a given sequence of tokens (see Seq).
one_of	one_of('0'..='9'), one_of("<>")	Recognises any single token that is part of a given sequence of tokens (see Seq).
any	any().filter(char::is_whitespace)	Recognises any single token, but not the end of the input.
todo()	foo.then(todo())	A placeholder parser that panics when invoked. Spiritually similar to the todo! macro.
custom	(see documentation for examples)	Allows implementing custom parsing logic, see the documentation for more information about how to write custom parsers.
end	x.then(end())	Recognises only the end of the input. Not to be confused with empty.
empty()	empty().then(y)	Recognises no input (i.e: it will always succeed, without advancing the input). Not to be confused with end.
Combinators
Combinators allow parsers to be combined together to make larger parsers. You can think of them as ‘parser operators’.

Because there are rather a lot of combinators, this section is split into categories to make finding the combinator you’re looking for easier.

Combining parsers
Combinators that allow combining smaller parsers together to make parsers for more complex grammars.

Name	Example	Description
Parser::then	a.then(b)	Parse one pattern and then another, producing a tuple of the two parsers outputs as an output.
Parser::or	a.or(b)	Parse one pattern, or another if the first failed to parse. This allows you to implement branching parser logic that recognises one of many different patterns.
Parser::ignore_then	a.ignore_then(b)	Parse one pattern and then another, producing only the output of the second as an output (i.e: ignoring the output of the first).
Parser::then_ignore	a.then_ignore(b)	Parse one pattern and then another, producing only the output of the first as an output (i.e: ignoring the output of the second).
Parser::delimited_by	a.delimited_by(x, y)	Parses a pattern, delimited by two other patterns on either side. Most often used to parse parenthesiesed expressions, blocks, or arrays.
Parser::padded_by	a.padded_by(b)	Parses a pattern, delimited by a pattern on either side. Often used to consume whitespace or other irrelevant input that surrounds a pattern.
Parser::repeated	a.repeated().collect::<Vec<_>>()	Parse the given pattern any number of times (including none at all!). Note that Repeated implements the IterParser trait, so can be used with IterParser::collect.
Parser::separated_by	a.separated_by(b).count()	Parses a pattern many times, interspersed with another pattern. Commonly used to parse things like comma-separated lists. Like Repeated, SeparatedBy implements IterParser.
Parser::or_not	a.or_not()	Attempt to parse a pattern, always succeeding with either [Some(...)] or None depending on whether parsing was successful. Can be used to optionally parse patterns.
Parser::foldl	a.foldl(b.repeated(), ...)	Parses a pattern, and then folds an IterParser into its output using the given function. Often used to parse binary operators.
IterParser::foldr	a.repeated().foldr(b, ...)	Parses elements of an IterParser, then a second pattern, folding the elements into the second parser’s output. Often used to parse unary operators.
Generating and manipulating outputs
Combinators that manipulate, generate, or combine the output of parsers in some manner (see backtracking and input manipulation for combinators that recover from errors).

Name	Example	Description
Parser::map	a.map(...)	Map the output of a parser using the given mapping function.
Parser::map_with	a.map_with(...)	Map the output of a parser using the given mapping function, with access to metadata associated with the output.
Parser::to_slice	a.to_slice()	Parse a pattern. Discard the output of the pattern and instead use a slice of the input that the pattern corresponds to as the output. Requires inputs that implement SliceInput.
Parser::to	a.to(x)	Parse a pattern, ignoring the output value and using a constant value as the output value instead.
Parser::ignored	a.ignored()	Parse a pattern, ignoring the output value and using [()] as the output value instead.
IterParser::collect	a.repeated().collect::<Vec<_>>()	Collects elements of an IterParser into a type implementing Container.
IterParser::collect_exactly	a.repeated().collect::<Vec<_>>()	Collects elements of an IterParser into an exact-sized type implementing ContainerExactly.
IterParser::count	a.repeated().count()	Count the number of elements produced by an IterParser.
Parser::unwrapped	a.unwrapped()	Parse a pattern that returns either a Result or an Option, then unwrap them.
Handling and emitting errors
Combinators that manipulate or emit errors, along with fallibly validating parser outputs.

Name	Example	Description
Parser::map_err	a.map_err(...)	Parse a pattern. On failure, map the parser error to another value. Often used to customise error messages or add extra information to them.
Parser::map_err_with_state	a.lazy()	Like Parser::map_err, but provides access to the parser state (see Parser::parse_with_state for more information).
Parser::try_map	a.try_map(...)	Map the output of a parser using the given fallible mapping function. If the function produces an error, the parser fails with that error.
Parser::try_map_with	a.try_map_with(...)
Parser::validate	a.validate(...)	Parse a pattern. On success, map the output to another value with the opportunity to emit extra secondary errors. Commonly used to check the validity of patterns in the parser.
Parser::filter	any().filter(char::is_lowercase)	Parse a pattern and apply the given filtering function to the output. If the filter function returns false, the parser fails.
Parser::labelled	a.labelled("a")	Parse a pattern, labelling it. What exactly this does depends on the error type, but it is generally used to give a pattern a more general name (for example, “expression”).
Text-oriented parsing
Combinators intended only for the parsing and manipulation of text-like inputs.

Name	Example	Description
Parser::padded	a.padded()	Skips whitespace on either side of a pattern for text-like inputs specifically (i.e: those with u8 or char tokens). A more specialised version of Parser::padded_by.
Parser::from_str	just("true").from_str().unwrapped()	Parse a pattern that outputs a string, then use Rust’s FromStr trait to parse it. Often paired with Parser::unwrapped to unwrap any errors.
Utility and error recovery
Miscellaneous combinators and those that relate to error recovery.

Name	Example	Description
Parser::boxed	a.boxed()	Performs type-erasure on a parser, allocating it on the heap. Stategically boxing of parsers can improve compilation times and allows dynamically building up parsers at runtime.
Parser::recover_with	a.recover_with(r)	Attempt to parse a pattern. On failure, the given recovery strategy is used to attempt to recovery from the error. See the documentation for more information.
Parser::memoized	a.memoized()	Parse a pattern, but remember whether it succeeded or failed and reuse that information when parsing the same input again. Allows expressing left-recursive or exponential patterns.
Backtracking and input manipulation
Combinators that perform internal backtracking or that manipulate inputs in order to function.

Name	Example	Description
Parser::not	a.and_is(b.not())	Doesn’t parse anything, but rejects anything that would parse as the given pattern. On success, no input is consumed.
Parser::and_is	a.and_is(b)	Parses one pattern, but only if the other parser also parses at the same location.
Parser::rewind	a.then(b.rewind())	Parses a pattern. On success, rewinds the input to the start of the pattern as if it had never been parsed. Often used parse patterns that expect to have something else after them
Parser::lazy	a.lazy()	Only useful on ‘top-level’ parsers. Makes the parser lazy such that it will only recognise as much input as it can and no more.
Parser::nested_in	a.nested_in(b)	Parse one pattern from the output of another pattern, using the output of the second parser as the input of the first. Often used to pass token trees.
Context-sensitive parsing
Combinators that allow the parsing of context-sensitive grammars (usually beyond the capability of top-down parsers).

Name	Example	Description
Parser::ignore_with_ctx	a.ignore_with_ctx(b)	Parse one pattern and use its output as context for another pattern, doesn’t return the context. See ConfigParser::configure for information about context-sensitive parsing.
Parser::then_with_ctx	a.then_with_ctx(b)	Parse one pattern and use its output as context for another pattern. returns the context. See ConfigParser::configure for information about context-sensitive parsing.
Parser::with_ctx	a.with_ctx(ctx)	Parse a pattern with the provided context.
See ConfigParser::configure for information about context-sensitive parsing.

# Recursion

Recursion
Most non-trivial languages - both spoken and programmed - are recursive. Grammars that describe these languages can express recursion by having a term in the language contain itself (either directly or indirectly). Noam Chomsky believed that recursion was so fundamental to human language that he considered it the primary demarcation between human and non-human language. This is debated in academic circles, but chumsky treats recursion with similar reverance.

The Problem
In Rust, writing a recursive function is usually trivial.

fn factorial(x: u32) -> u32 {
    if x <= 1 {
        1
    } else {
        x * factorial(x - 1)
    }
}
However, chumsky parsers are values, not functions. Just like Iterators, they can be moved around, manipulated, and invoked in a lazy manner. Intuitively, we might think to write a recursive parser to parse 4 + (1 + 2) + 3 like so:

use chumsky::prelude::*;

fn a_parser<'src>() -> impl Parser<'src, &'src str, i32> + Clone {
    let int = text::int(10).map(|s: &str| s.parse().unwrap());

    let atom = choice((
        int,
        a_parser().delimited_by(just('('), just(')')),
    ))
        .padded();

    atom.clone().foldl(
        just('+').padded().ignore_then(atom).repeated(),
        |lhs, rhs| lhs + rhs,
    )
}
Unfortunately, we hit an error:

error[E0720]: cannot resolve opaque type
   --> recursion.rs:1:24
    |
 1  |   fn a_parser<'src>() -> impl Parser<'src, &'src str, i32> + Clone {
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ recursive opaque type
...
 9  | /     atom.clone().foldl(
10  | |         just('+').padded().ignore_then(atom).repeated(),
11  | |         |lhs, rhs| lhs + rhs,
12  | |     )
    | |     -
    | |_____|
    | |_____returning here with type `...`
We can ‘solve’ this problem by boxing a_parser(), but all it does is convert the compilation error into a run-time stack overflow. Why? The answer, if we take a step back, should be obvious: our a_parser function isn’t actually doing any parsing, it’s just creating a parser. In order to create a parser, it needs to call itself… which means calling itself again… forever. We’ve created infinite recursion. No dice.

A Solution
To get us out of this somewhat sticky bind, chumsky provides a special combinator called recursive. I allows us to refer to a parser within its own definition - without getting us caught in recursive hot water.

use chumsky::prelude::*;

fn a_parser<'src>() -> impl Parser<'src, &'src str, i32> {
    recursive(|a_parser| {
        let int = text::int(10).map(|s: &str| s.parse().unwrap());

        let atom = choice((
            int,
            a_parser.delimited_by(just('('), just(')')),
        ))
            .padded();

        atom.clone().foldl(
            just('+').padded().ignore_then(atom).repeated(),
            |lhs, rhs| lhs + rhs,
        )
    })
}
Notice how our a_parser function is no longer recursive: instead, we get the definition of a_parser from the closure parameter.

More Complicated Cases
More complicated parsers tend to have many mutually-recursive patterns. For example, in Rust’s syntax, the ‘expression’ and ‘type’ terms are intertwined: expressions can contain types (in the form of turbofish type annotations, or in as casts) and types can contain expressions (in array type sizes or in const generics).

It is possible to use recursive in a ‘nested’ manner to express such a thing,
but chumsky provides a simpler solution: Recursive::declare and
Recursive::define. These functions allow us to entirely decouple the declaration
and definition of a recursive parser, giving us the ability to easily declare
our mutually-recursive parsers up-front and then use them in each other’s
definitions.

# Technical Notes

Technical Notes
This section contains assorted details about chumsky. Most of this information is irrelevant to beginners, but we consider it important enough to include for advanced users.

Technical Notes
Classification
Purity and optimisation
Classification
Chumsky is a PEG parser by nature. That is to say, it is possible to parse all known context-free grammars with chumsky. It has not yet been formally proven that PEG parsers can parse all context-free grammars but, for the sake of using the library, it is reasonable to assume as much.

Chumsky also has limited support for context-sensitive parsing. Chumsky’s context-sensitive parsing allows previously parsed elements of the grammar to inform the parsing of future elements in a limited way. See Parser::ignore_with_ctx and Parser::then_with_ctxfor more information.

The term ‘PEG++’ might be an appropriate description of chumsky, with ‘CFG + left context’ being a description of the grammars that it can parse.

Chumsky can also be extended via custom and ExtParser, permitting it to theoretically parse any parseable grammar: but this is probably cheating since doing so requires manually implementing such parser logic.

Purity and optimisation
Chumsky uses a plethora of techniques to improve parser performance. For example, it may skip generating output values that go unused by the parser (such as the output of a in a.ignore_then(b)). This also includes combinators like Parser::map, which accept a user-provided closure. However, chumsky has no control over the behaviour of this closure, and it’s possible to observe the closure being ‘optimised away’.

For this reason, unless otherwise specified, any closures/functions used inline
within a chumsky parser should be semantically pure: that is, you should not
assume that they are called any specific number of times. This does not mean
that they are not permitted to have side effects, but that those side effects
should be irrelevant to the correct functioning of the parser. For example,
string interning within Parser::map_with is an impure operation, but this
impurity does not affect the correct functioning of the parser: interning a
string that goes unused can be done any number of times or not at all without
resulting in bad behaviour.

# Chumsky: A Tutorial

In this tutorial, we’ll develop a parser (and interpreter!) for a programming language called ‘Foo’.

Foo is a small language, but it’s enough for us to have some fun. It isn’t Turing-complete, but it is complex enough to allow us to get to grips with parsing using Chumsky, containing many of the elements you’d find in a ‘real’ programming language. Here’s some sample code written in Foo:

let seven = 7;
fn add x y = x + y;
add(2, 3) * -seven
By the end of this tutorial, you’ll have an interpreter that will let you run this code, and more.

This tutorial should take somewhere between 30 and 100 minutes to complete, depending on factors such as knowledge of Rust and compiler theory.

You can find the source code for the full interpreter in examples/foo.rs in the main repository.

Assumptions
This tutorial is here to show you how to use Chumsky: it’s not a general-purpose introduction to language development as a whole. For that reason, we make a few assumptions about things you should know before jumping in:

You should be happy reading and writing Rust. Particularly obscure syntax will be explained, but you should already be reasonably confident with concepts like functions, types, pattern matching, and error handling (Result, ?, etc.).
You should be familiar with data structures like trees and vectors.
You should have some awareness of basic compiler theory concepts like Abstract Syntax Trees (ASTs), the difference between parsing and evaluation, Backus Naur Form (BNF), etc.
Documentation
As we go, we’ll be encountering many functions and concepts from Chumsky. I strongly recommend you keep Chumsky’s documentation open in another browser tab and use it to cross-reference your understanding or gain more insight into specific things that you’d like more clarification on. In particular, most of the functions we’ll be using come from the Parser trait. Chumsky’s docs include extensive doc examples for almost every function, so be sure to make use of them!

Chumsky also has several longer examples in the main repository: looking at these may help improve your understanding if you get stuck.

A note on imperative vs declarative parsers
If you’ve tried hand-writing a parser before, you’re probably expecting lots of flow control: splitting text by whitespace, matching/switching/branching on things, making a decision about whether to recurse into a function or expect another token, etc. This is an imperative approach to parser development and can be very time-consuming to write, maintain, and test.

In contrast, Chumsky parsers are declarative: they still perform intricate flow control internally, but it’s all hidden away so you don’t need to think of it. Instead of describing how to parse a particular grammar, Chumsky parsers simply describe a grammar: and it is then Chumsky’s job to figure out how to efficiently parse it.

If you’ve ever seen Backus Naur Form (BNF) used to describe a language’s syntax, you’ll have a good sense of what this means: if you squint, you’ll find that a lot of parsers written in Chumsky look pretty close to the BNF definition.

Another consequence of creating parsers in a declarative style is that defining a parser and using a parser are two different things: once created, parsers won’t do anything on their own unless you give them an input to parse.

Similarities between Parser and Iterator
The most important API in Chumsky is the Parser trait, implemented by all parsers. Because parsers don’t do anything by themselves, writing Chumsky parsers often feels very similar to writing iterators in Rust using the Iterator trait. If you’ve enjoyed writing iterators in Rust before, you’ll hopefully find the same satisfaction writing parsers with Chumsky. They even share several functions with each other!

Setting up
Create a new project with cargo new --bin foo, add the latest version of Chumsky as a dependency, and place the following in your main.rs:

ⓘ
use chumsky::prelude::*;

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    println!("{}", src);
}
This code has one purpose: it treats the first command-line argument as a path, reads the corresponding file, then prints the contents to the terminal. We don’t really care for handling IO errors in this tutorial, so .unwrap() will suffice.

Create a file named test.foo and run cargo run -- test.foo (the -- tells cargo to pass the remaining arguments to the program instead of cargo itself). You should see that the contents of test.foo, if any, get printed to the console.

Next, we’ll create a data type that represents a program written in Foo. All programs in Foo are expressions, so we’ll call it Expr.

ⓘ
#[derive(Debug)]
enum Expr<'a> {
    Num(f64),
    Var(&'a str),

    Neg(Box<Expr<'a>>),
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    Sub(Box<Expr<'a>>, Box<Expr<'a>>),
    Mul(Box<Expr<'a>>, Box<Expr<'a>>),
    Div(Box<Expr<'a>>, Box<Expr<'a>>),

    Call(&'a str, Vec<Expr<'a>>),
    Let {
        name: &'a str,
        rhs: Box<Expr<'a>>,
        then: Box<Expr<'a>>,
    },
    Fn {
        name: &'a str,
        args: Vec<&'a str>,
        body: Box<Expr<'a>>,
        then: Box<Expr<'a>>,
    }
}
This is Foo’s Abstract Syntax Tree (AST). It represents all possible Foo programs and is defined recursively in terms of itself (Box is used to avoid the type being infinitely large). Each expression may itself contain sub-expressions.

As an example, the expression let x = 5; x * 3 is encoded as follows using the Expr type:

ⓘ
Expr::Let {
    name: "x",
    rhs: Box::new(Expr::Num(5.0)),
    then: Box::new(Expr::Mul(
        Box::new(Expr::Var("x")),
        Box::new(Expr::Num(3.0))
    )),
}
The purpose of our parser will be to perform this conversion, from source code to AST.

We’re also going to create a function that creates Foo’s parser. Our parser takes in a char stream and produces an Expr, so we’ll use those types for the I (input) and O (output) type parameters.

ⓘ
fn parser<'a>() -> impl Parser<'a, &'a str, Expr<'a>> {
    // To be filled in later...
}
In main, we’ll alter the println! as follows:

ⓘ
println!("{:?}", parser().parse(&src));
Parsing digits
Chumsky is a ‘parser combinator’ library. It allows the creation of parsers by combining together many smaller parsers. The very smallest parsers are called ‘primitives’ and live in the primitive module.

We’re going to want to start by parsing the simplest element of Foo’s syntax: numbers.

ⓘ
// In `parser`...
any()
    .filter(|c: &char| c.is_ascii_digit())
The any primitive accepts any token(except the EOF) from str, then pass each token(a char) to next primitive filter.

The filter primitive allows us to read a single input and accept it if it passes a condition. In our case, that condition simply checks that the character is a digit.

If we compile this code now, we’ll encounter an error. Why?

Although we promised that our parser would produce an Expr, the filter primitive only outputs the input it found. Right now, all we have is a parser from str to char instead of a parser from str to Expr!

To solve this, we need to crack open the ‘combinator’ part of parser combinators. We’ll use Chumsky’s map method to convert the output of the parser to an Expr. This method is very similar to its namesake on Iterator.

ⓘ
any()
    .filter(|c: &char| c.is_ascii_digit())
    .map(|c| Expr::Num(c.to_digit(10).unwrap() as f64))
Here, we’re converting the char digit to an f64 (unwrapping is fine: map only gets applied to outputs that successfully parsed!) and then wrapping it in Expr::Num(_) to convert it to a Foo expression.

Try running the code. You’ll see that you can type a digit into test.foo and have our interpreter generate an AST like so:

ⓘ
ParseResult { output: Some(Num(5.0)), errs: [] }
Parsing numbers
If you’re more than a little adventurous, you’ll quickly notice that typing in a multi-digit number doesn’t quite behave as expected. Inputting 42 produces a None output:

ⓘ
ParseResult { output: None, errs: [EmptyErr(())] }
This is because by default Chumsky’s parsers are NOT lazy, that means a parser will produce an error if all input is not consumed, this is what we expected for most parsers.

Combining these together, we now get an error for longer inputs. Unfortunately, this just reveals another problem (particularly if you’re working on a Unix-like platform): any whitespace before or after our digit will upset our parser and trigger an error.

We can handle whitespace by adding a call to padded_by (which ignores a given pattern before and after the first) after our digit parser, and a repeating filter for any whitespace characters.

ⓘ
any()
    .filter(|c: &char| c.is_ascii_digit())
    .map(|c| Expr::Num(c.to_digit(10).unwrap() as f64))
    .padded_by(any().filter(|c: &char| c.is_whitespace()).repeated())
This example should have taught you a few important things about Chumsky’s parsers:

Parsers are NOT lazy: all input must be consumed
Whitespace is not automatically ignored. Chumsky is a general-purpose parsing library, and some languages care very much about the structure of whitespace, so Chumsky does too
Cleaning up and taking shortcuts
At this point, things are starting to look a little messy. We’ve ended up writing 4 lines of code to properly parse a single digit. Let’s clean things up a bit. We’ll also make use of a bunch of text-based parser primitives that come with Chumsky to get rid of some of this cruft.

ⓘ
let int = text::int(10)
    .map(|s: &str| Expr::Num(s.parse().unwrap()))
    .padded();
int // `int` will be used later, bear with it now
text::int(10) accepts decimal integers (10 is the base); map is still used but now it can parse multiple digits; padded is a shortcut for ignoring whitespaces.

That’s better. We’ve also swapped out our custom digit parser with a built-in parser that parses any non-negative integer.

Evaluating simple expressions
We’ll now take a diversion away from the parser to create a function that can evaluate our AST. This is the ‘heart’ of our interpreter and is the thing that actually performs the computation of programs.

ⓘ
fn eval<'a>(expr: &'a Expr<'a>) -> Result<f64, String> {
    match expr {
        Expr::Num(x) => Ok(*x),
        Expr::Neg(a) => Ok(-eval(a)?),
        Expr::Add(a, b) => Ok(eval(a)? + eval(b)?),
        Expr::Sub(a, b) => Ok(eval(a)? - eval(b)?),
        Expr::Mul(a, b) => Ok(eval(a)? * eval(b)?),
        Expr::Div(a, b) => Ok(eval(a)? / eval(b)?),
        _ => todo!(), // We'll handle other cases later
    }
}
This function might look scary at first glance, but there’s not too much going on here: it just recursively calls itself, evaluating each node of the AST, combining the results via operators, until it has a final result. Any runtime errors simply get thrown back down the stack using ?.

We’ll also change our main function a little so that we can pass our AST to eval.

ⓘ
fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    match parser().parse(&src).into_result() {
        Ok(ast) => match eval(&ast) {
            Ok(output) => println!("{}", output),
            Err(eval_err) => println!("Evaluation error: {}", eval_err),
        },
        Err(parse_errs) => parse_errs
            .into_iter()
            .for_each(|e| println!("Parse error: {}", e)),
    }
}
This looks like a big change, but it’s mostly just an extension of the previous code to pass the AST on to eval if parsing is successful. If unsuccessful, we just print the errors generated by the parser. Right now, none of our operators can produce errors when evaluated, but this will change in the future so we make sure to handle them in preparation.

Parsing unary operators
Jumping back to our parser, let’s handle unary operators. Currently, our only unary operator is -, the negation operator. We’re looking to parse any number of -, followed by a number. More formally:

expr = op* + int
We’ll also give our int parser a new name, atom, for reasons that will become clear later.

ⓘ
let int = text::int(10)
    .map(|s: &str| Expr::Num(s.parse().unwrap()))
    .padded();

let atom = int;

let op = |c| just(c).padded();

let unary = op('-')
    .repeated()
    .foldr(atom, |_op, rhs| Expr::Neg(Box::new(rhs)));

unary
Here, we meet a few new combinators:

just defines a parser that accepts only the given input. We leverage it to define op that can easily construct an operator parser later by passing the operator character.

repeated will parse a given pattern any number of times (including zero!).

foldr means “right-fold”, it iterates the preceding output(provided by repeated), fold all values into a single value by repeatedly applying the given closure. The first argument atom provides the initial value of the folding process.

This is worth a little more consideration. We’re trying to parse any number of negation operators, followed by a single atom (for now, just a number). For example, the input ---42 would generate the following input to foldr:

ⓘ
(['-', '-', '-'], Num(42.0))
The foldr function repeatedly applies the function to ‘fold’ the elements into a single element, like so:

(['-',   '-',   '-'],   Num(42.0))
  ---    ---    ---     ---------
   |      |      |           |
   |      |       \         /
   |      |      Neg(Num(42.0))
   |      |            |
   |       \          /
   |    Neg(Neg(Num(42.0)))
   |            |
    \          /
Neg(Neg(Neg(Num(42.0))))
This may be a little hard to conceptualise for those used to imperative programming, but for functional programmers it should come naturally: foldr is just equivalent to reduce!

Give the interpreter a try. You’ll be able to enter inputs as before, but also values like -17. You can even apply the negation operator multiple times: --9 will yield a value of 9 in the command line.

This is exciting: we’ve finally started to see our interpreter perform useful (sort of) computations!

Parsing binary operators
Let’s keep the momentum going and move over to binary operators. Traditionally, these pose quite a problem for parsers. To parse an expression like 3 + 4 * 2, it’s necessary to understand that multiplication binds more eagerly than addition and hence is applied first. Therefore, the result of this expression is 11 and not 14.

Parsers employ a range of strategies to handle these cases, but for Chumsky things are simple: the most eagerly binding (highest ‘precedence’) operators should be those that get considered first when parsing.

It’s worth noting that summation operators (+ and -) are typically considered to have the same precedence as one-another. The same also applies to product operators (* and /). For this reason, we treat each group as a single pattern.

At each stage, we’re looking for a simple pattern: an unary expression, following by any number of a combination of an operator and an unary expression. More formally:

expr = unary + (op + unary)*
Let’s expand our parser.

ⓘ
let int = text::int(10)
    .map(|s: &str| Expr::Num(s.parse().unwrap()))
    .padded();

let atom = int;

let op = |c| just(c).padded();

let unary = op('-')
    .repeated()
    .foldr(atom, |_op, rhs| Expr::Neg(Box::new(rhs)));

let product = unary.foldl(
    choice((
        op('*').to(Expr::Mul as fn(_, _) -> _),
        op('/').to(Expr::Div as fn(_, _) -> _),
    ))
    .then(unary)
    .repeated(),
    |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
);

let sum = product.foldl(
    choice((
        op('+').to(Expr::Add as fn(_, _) -> _),
        op('-').to(Expr::Sub as fn(_, _) -> _),
    ))
    .then(product)
    .repeated(),
    |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
);

sum
The Expr::Mul as fn(_, _) -> _ syntax might look a little unfamiliar, but don’t worry! In Rust, tuple enum variants are implicitly functions. All we’re doing here is making sure that Rust treats each of them as if they had the same type using the as cast, and then letting type inference do the rest. Those functions then get passed through the internals of the parser and end up in op within the foldl call.

Another three combinators are introduced here:

choice attempts each parser in a tuple, producing the output of the first to successfully parse

to is similar to map, but instead of mapping the output, entirely overrides the output with a new value. In our case, we use it to convert each binary operator to a function that produces the relevant AST node for that operator.

foldl is very similar to foldr in the last section but, instead of operating on a (Vec<_>, _), it operates upon a (_, Vec<_>), going backwards to combine values together with the function

In a similar manner to foldr in the previous section on unary expressions, foldl is used to fold chains of binary operators into a single expression tree. For example, the input 2 + 3 - 7 + 5 would generate the following input to foldl:

ⓘ
(Num(2.0), [(Expr::Add, Num(3.0)), (Expr::Sub, Num(7.0)), (Add, Num(5.0))])
This then gets folded together by foldl like so:

(Num(2.0),   [(Add, Num(3.0)),   (Sub, Num(7.0)),   (Add, Num(5.0))])
 --------     ---------------     --------------    ---------------
    |                |                 |                  |
     \              /                  |                  |
 Add(Num(2.0), Num(3.0))               |                  |
            |                          |                  |
             \                        /                   |
      Sub(Add(Num(2.0), Num(3.0)), Num(7.0))              |
                       |                                  |
                        \                                /
               Add(Sub(Add(Num(2.0), Num(3.0)), Num(7.0)), Num(5.0))
Give the interpreter a try. You should find that it can correctly handle both unary and binary operations combined in arbitrary configurations, correctly handling precedence. You can use it as a pocket calculator!

Parsing parentheses
A new challenger approaches: nested expressions. Sometimes, we want to override the default operator precedence rules entirely. We can do this by nesting expressions within parentheses, like (3 + 4) * 2. How do we handle this?

The creation of the atom pattern a few sections before was no accident: parentheses have a greater precedence than any operator, so we should treat a parenthesized expression as if it were equivalent to a single value. We call things that behave like single values ‘atoms’ by convention.

We’re going to hoist our entire parser up into a closure, allowing us to define it in terms of itself.

ⓘ
recursive(|expr| {
    let int = text::int(10).map(|s: &str| Expr::Num(s.parse().unwrap()));

    let atom = int.or(expr.delimited_by(just('('), just(')'))).padded();

    let op = |c| just(c).padded();

    let unary = op('-')
        .repeated()
        .foldr(atom, |_op, rhs| Expr::Neg(Box::new(rhs)));

    let product = unary.clone().foldl(
        choice((
            op('*').to(Expr::Mul as fn(_, _) -> _),
            op('/').to(Expr::Div as fn(_, _) -> _),
        ))
        .then(unary)
        .repeated(),
        |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
    );

    let sum = product.clone().foldl(
        choice((
            op('+').to(Expr::Add as fn(_, _) -> _),
            op('-').to(Expr::Sub as fn(_, _) -> _),
        ))
        .then(product)
        .repeated(),
        |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
    );

    sum
})
There are a few things worth paying attention to here.

or attempts to parse a pattern and, if unsuccessful, instead attempts another pattern

recursive allows us to define a parser recursively in terms of itself by giving us a copy of it within the closure’s scope

We use the recursive definition of expr within the definition of atom. We use the new delimited_by combinator to allow it to sit nested within a pair of parentheses

We have to clone unary and product to use them in the closure of recursive

Try running the interpreter. You’ll find that it can handle a surprising number of cases elegantly. Make sure that the following cases work correctly:

Expression	Expected result
3 * 4 + 2	14
3 * (4 + 2)	18
-4 + 2	-2
-(4 + 2)	-6
Parsing lets
Our next step is to handle let. Unlike Rust and other imperative languages, let in Foo is an expression and not an statement (Foo has no statements) that takes the following form:

ⓘ
let <ident> = <expr>; <expr>
We only want lets to appear at the outermost level of the expression, so we leave it out of the original recursive expression definition. However, we also want to be able to chain lets together, so we put them in their own recursive definition. We call it decl (‘declaration’) because we’re eventually going to be adding fn syntax too.

ⓘ
let ident = text::ascii::ident().padded();

let expr = recursive(|expr| {
    let int = text::int(10).map(|s: &str| Expr::Num(s.parse().unwrap()));

    let atom = int
        .or(expr.delimited_by(just('('), just(')')))
        .or(ident.map(Expr::Var))
        .padded();

    let op = |c| just(c).padded();

    let unary = op('-')
        .repeated()
        .foldr(atom, |_op, rhs| Expr::Neg(Box::new(rhs)));

    let product = unary.clone().foldl(
        choice((
            op('*').to(Expr::Mul as fn(_, _) -> _),
            op('/').to(Expr::Div as fn(_, _) -> _),
        ))
        .then(unary)
        .repeated(),
        |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
    );

    let sum = product.clone().foldl(
        choice((
            op('+').to(Expr::Add as fn(_, _) -> _),
            op('-').to(Expr::Sub as fn(_, _) -> _),
        ))
        .then(product)
        .repeated(),
        |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
    );

    sum
});

let decl = recursive(|decl| {
    let r#let = text::ascii::keyword("let")
        .ignore_then(ident)
        .then_ignore(just('='))
        .then(expr.clone())
        .then_ignore(just(';'))
        .then(decl)
        .map(|((name, rhs), then)| Expr::Let {
            name,
            rhs: Box::new(rhs),
            then: Box::new(then),
        });

    r#let
        // Must be later in the chain than `r#let` to avoid ambiguity
        .or(expr)
        .padded()
});

decl
keyword is simply a parser that looks for an exact identifier (i.e: it doesn’t match identifiers that only start with a keyword).

Other than that, there’s nothing in the definition of r#let that you haven’t seen before: familiar combinators, but combined in different ways. It selectively ignores parts of the syntax that we don’t care about after validating that it exists, then uses those elements that it does care about to create an Expr::Let AST node.

Another thing to note is that the definition of ident will parse "let". To avoid the parser accidentally deciding that "let" is a variable, we place r#let earlier in the or chain than expr so that it priorities the correct interpretation. As mentioned in previous sections, Chumsky handles ambiguity simply by choosing the first successful parse it encounters, so making sure that we declare things in the right order can sometimes be important.

You should now be able to run the interpreter and have it accept an input such as

ⓘ
let five = 5;
five * 3
Unfortunately, the eval function will panic because we’ve not yet handled Expr::Var or Expr::Let. Let’s do that now.

ⓘ
fn eval<'a>(expr: &'a Expr<'a>, vars: &mut Vec<(&'a str, f64)>) -> Result<f64, String> {
    match expr {
        Expr::Num(x) => Ok(*x),
        Expr::Neg(a) => Ok(-eval(a, vars)?),
        Expr::Add(a, b) => Ok(eval(a, vars)? + eval(b, vars)?),
        Expr::Sub(a, b) => Ok(eval(a, vars)? - eval(b, vars)?),
        Expr::Mul(a, b) => Ok(eval(a, vars)? * eval(b, vars)?),
        Expr::Div(a, b) => Ok(eval(a, vars)? / eval(b, vars)?),
        Expr::Var(name) => {
            if let Some((_, val)) = vars.iter().rev().find(|(var, _)| var == name) {
                Ok(*val)
            } else {
                Err(format!("Cannot find variable `{}` in scope", name))
            }
        }
        Expr::Let { name, rhs, then } => {
            let rhs = eval(rhs, vars)?;
            vars.push((name, rhs));
            let output = eval(then, vars);
            vars.pop();
            output
        }
        _ => todo!(),
    }
}
Woo! That got a bit more complicated. Don’t fear, there are only 3 important changes:

Because we need to keep track of variables that were previously defined, we use a Vec to remember them. Because eval is a recursive function, we also need to pass is to all recursive calls.

When we encounter an Expr::Let, we first evaluate the right-hand side (rhs). Once evaluated, we push it to the vars stack and evaluate the trailing then expression (i.e: all of the remaining code that appears after the semicolon). Popping it afterwards is not technically necessary because Foo does not permit nested declarations, but we do it anyway because it’s good practice and it’s what we’d want to do if we ever decided to add nesting.

When we encounter an Expr::Var (i.e: an inline variable) we search the stack backwards (because Foo permits variable shadowing and we only want to find the most recently declared variable with the same name) to find the variable’s value. If we can’t find a variable of that name, we generate a runtime error which gets propagated back up the stack.

Obviously, the signature of eval has changed so we’ll update the call in main to become:

ⓘ
eval(&ast, &mut Vec::new())
Make sure to test the interpreter. Try experimenting with let declarations to make sure things aren’t broken. In particular, it’s worth testing variable shadowing by ensuring that the following program produces 8:

ⓘ
let x = 5;
let x = 3 + x;
x
Parsing functions
We’re almost at a complete implementation of Foo. There’s just one thing left: functions.

Surprisingly, parsing functions is the easy part. All we need to modify is the definition of decl to add r#fn. It looks very much like the existing definition of r#let:

ⓘ
let decl = recursive(|decl| {
    let r#let = text::ascii::keyword("let")
        .ignore_then(ident)
        .then_ignore(just('='))
        .then(expr.clone())
        .then_ignore(just(';'))
        .then(decl.clone())
        .map(|((name, rhs), then)| Expr::Let {
            name,
            rhs: Box::new(rhs),
            then: Box::new(then),
        });

    let r#fn = text::ascii::keyword("fn")
        .ignore_then(ident)
        .then(ident.repeated().collect::<Vec<_>>())
        .then_ignore(just('='))
        .then(expr.clone())
        .then_ignore(just(';'))
        .then(decl)
        .map(|(((name, args), body), then)| Expr::Fn {
            name,
            args,
            body: Box::new(body),
            then: Box::new(then),
        });

    r#let.or(r#fn).or(expr).padded()
});
The only thing to note here, is the repeated(), which gives us an IterParser, we have to call collect on it to collect the output elements into a collection.

Obviously, we also need to add support for calling functions by modifying atom:

ⓘ
 let call = ident
    .then(
        expr.clone()
            .separated_by(just(','))
            .allow_trailing()   // Foo is Rust-like, so allow trailing commas to appear in arg lists
            .collect::<Vec<_>>()
            .delimited_by(just('('), just(')')),
    )
    .map(|(f, args)| Expr::Call(f, args));

let atom = int
    .or(expr.delimited_by(just('('), just(')')))
    .or(call)
    .or(ident.map(Expr::Var))
    .padded();
The only new combinator here is separated_by which behaves like repeated, but requires a separator pattern between each element. It has a method called allow_trailing which allows for parsing a trailing separator at the end of the elements.

Next, we modify our eval function to support a function stack.

ⓘ
fn eval<'a>(
    expr: &'a Expr<'a>,
    vars: &mut Vec<(&'a str, f64)>,
    funcs: &mut Vec<(&'a str, &'a [&'a str], &'a Expr<'a>)>,
) -> Result<f64, String> {
    match expr {
        Expr::Num(x) => Ok(*x),
        Expr::Neg(a) => Ok(-eval(a, vars, funcs)?),
        Expr::Add(a, b) => Ok(eval(a, vars, funcs)? + eval(b, vars, funcs)?),
        Expr::Sub(a, b) => Ok(eval(a, vars, funcs)? - eval(b, vars, funcs)?),
        Expr::Mul(a, b) => Ok(eval(a, vars, funcs)? * eval(b, vars, funcs)?),
        Expr::Div(a, b) => Ok(eval(a, vars, funcs)? / eval(b, vars, funcs)?),
        Expr::Var(name) => {
            if let Some((_, val)) = vars.iter().rev().find(|(var, _)| var == name) {
                Ok(*val)
            } else {
                Err(format!("Cannot find variable `{}` in scope", name))
            }
        }
        Expr::Let { name, rhs, then } => {
            let rhs = eval(rhs, vars, funcs)?;
            vars.push((*name, rhs));
            let output = eval(then, vars, funcs);
            vars.pop();
            output
        }
        Expr::Call(name, args) => {
            if let Some((_, arg_names, body)) =
                funcs.iter().rev().find(|(var, _, _)| var == name).copied()
            {
                if arg_names.len() == args.len() {
                    let mut args = args
                        .iter()
                        .map(|arg| eval(arg, vars, funcs))
                        .zip(arg_names.iter())
                        .map(|(val, name)| Ok((*name, val?)))
                        .collect::<Result<_, String>>()?;
                    let old_vars = vars.len();
                    vars.append(&mut args);
                    let output = eval(body, vars, funcs);
                    vars.truncate(old_vars);
                    output
                } else {
                    Err(format!(
                        "Wrong number of arguments for function `{}`: expected {}, found {}",
                        name,
                        arg_names.len(),
                        args.len(),
                    ))
                }
            } else {
                Err(format!("Cannot find function `{}` in scope", name))
            }
        }
        Expr::Fn {
            name,
            args,
            body,
            then,
        } => {
            funcs.push((name, args, body));
            let output = eval(then, vars, funcs);
            funcs.pop();
            output
        }
    }
}
Another big change! On closer inspection, however, this looks a lot like the change we made previously when we added support for let declarations. Whenever we encounter an Expr::Fn, we just push the function to the funcs stack and continue. Whenever we encounter an Expr::Call, we search the function stack backwards, as we did for variables, and then execute the body of the function (making sure to evaluate and push the arguments!).

As before, we’ll need to change the eval call in main to:

ⓘ
eval(&ast, &mut Vec::new(), &mut Vec::new())
Give the interpreter a test - see what you can do with it! Here’s an example program to get you started:

ⓘ
let five = 5;
let eight = 3 + five;
fn add x y = x + y;
add(five, eight)
Conclusion
Here ends our exploration into Chumsky’s API. We only scratched the surface of what Chumsky can do, but now you’ll need to rely on the examples in the repository and the API doc examples for further help. Nonetheless, I hope it was an interesting foray into the use of parser combinators for the development of parsers.

If nothing else, you’ve now got a neat little calculator language to play with.

Interestingly, there is a subtle bug in Foo’s eval function that produces unexpected scoping behaviour with function calls. I’ll leave finding it as an exercise for the reader.

Extension tasks
Find the interesting function scoping bug and consider how it could be fixed

Split token lexing into a separate compilation stage to avoid the need for .padded() in the parser

Add more operators

Add an if <expr> then <expr> else <expr> ternary operator

Add values of different types by turning f64 into an enum

Add lambdas to the language

Format the error message in a more useful way, perhaps by providing a reference to the original code