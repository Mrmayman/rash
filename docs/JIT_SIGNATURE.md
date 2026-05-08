The signature of the JIT-compiled function that runs Scratch code.

# Execution model

JIT functions behave similarly to coroutines. A function may pause in between execution, requiring you to call it again to resume it. It may either return:
- return [`JumpId::DONE`] to indicate completion
- return some other [`JumpId`] to indicate suspension

To resume execution, call the same function again with the returned [`JumpId`] and the preserved execution state.

Terminology:
- Yielding: The act of a function pausing and requiring you to resume it.
- **Custom Blocks**: Functions defined in the Scratch language. Not talking about "real" native functions here.
- **Warp functions**: Custom Blocks that don't yield (pause). Also known as "Run without Screen Refresh" enabled blocks.
- **Yielding functions**: Custom Blocks that do yield, AKA everything that doesn't enable "Run without Screen Refresh".

This terminology may not match scratch-specific terms, I'm using clearer ones.

Any function can indirectly inherit warp-ness (non-yielding) when called by a warp function.
# Safety

> Note: When I talk about safety here, I'm talking about memory safety, not logical/semantical correctness.

- **Safety in arguments:** You can pass `null` as any of the pointers, if you aren't using their specific features. If you passed `null` to something that was required, it will **safely panic**.
	- There is **one major exception:** *Custom Block arguments*. Make sure they're valid (see below docs for more details).
- **Lifetimes:** The VM's memory buffer that existed when this code was compiled, must stay alive for as long as the code will be called. The pointers are hardcoded into the machine code for performance.
	- Not really many other notes on lifetimes, other than this.

# Arguments

- [`JumpId`]: The execution state to resume from. Pass [`JumpId::default`] to start from beginning.
- `*mut Vec<LoopFrame>`: The loop stack, represents what loops we're inside, and how many times it iterated out of what total limit.
	- This is used for storing state between yields, so it can be `null` for warp functions.
- `*const ScratchObject`:  A list of arguments when a Scratch function ("Custom Block") is called.
	- Points to the first element of a contiguous array of [`ScratchObject`] values.
	- The compiled function accesses arguments through fixed offsets from this pointer.
	- **WARNING:** If the Custom Block requires arguments, this *must* be valid and have the right number of elements. There is no bounds checking for performance reasons.
		- If the Custom Block doesn't require arguments, it doesn't matter what you pass here, though.
- `*const Scripts`: Compiled functions ready to be spawned/executed.
	- This is used for "spawning" Custom Blocks to be called, ie. getting a handle to another JIT function to be called from a JIT function.
	- Can be `null` if you aren't calling any Custom Blocks.
- `i64`: Is yielding enabled (pausable)? (1 or 0)
	- (Also known as "Screen Refresh" in Scratch)
	- Default `1`. Opt in to false (`0`) for better performance if you know the functions won't yield.
	- This is used for propagating non-yielding behavior through a long chain of calls (see top of this doc, "Execution model").
- `*mut Option<ScratchThread>`: 
	- Place to store the state of any child function that is called by the parent.
	- Let's say we have a function `foo()` that calls `bar()`. If `bar()` yields while called by `foo()`, then `foo()` stores `bar()`'s [`ScratchThread`] inside this `Option` (`None` by default), before pausing itself. Then, on resume it recursively walks down this linked list of `ScratchThread`s until it finds the final element, the function to first resume.
	- Can be `null` if this function doesn't yield or doesn't call anything that yields.
