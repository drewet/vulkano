# Design document

The Vulkan specs are not public, but [the Mantle specs are](http://www.amd.com/Documents/Mantle-Programming-Guide-and-API-Reference.pdf). Many design decisions can be made already based on what Mantle shows us.

## Objectives of this library

 - Be safe.
 - Be convenient to use.
 - Don't hide what it's doing.

## Extensions

Mantle has an extensions system for things such as DMA, occlusion query or platform-specific bindings.
It is unclear how Vulkan handles this.

Mantle's bindings to Windows is pretty simple: you need to create a special kind of resource that is similar to images, and this special kind of resource can be drawn on a `HWND`. The image isn't tied to the `HWND` ; you need tell which `HWND` to use each time you swap buffers. Deciding how to bind this library to platform-specific interfaces requires a broader picture.

## Thread safety

Some Mantle functions are marked as not being thread-safe. However we most likely want to store them in `Arc`s.

If we used `&mut` for non-thread-safe operations, we would need to wrap the entire object in a `Mutex`. Instead the best solution is to use an internal mutex that is used only for thread-unsafe operations:

```rust
pub struct Device {
    mutex: Mutex<()>,
    ...
}

impl Device {
    pub fn thread_safe_operation(&self) {
        // mutex not used
        vkDoSomething();
    }

    pub fn not_thread_safe_operation(&self) {
        let _lock = self.mutex.lock();
        vkDoSomethingElse();
    }
}
```

## Shaders introspection

When you use a descriptor set so that it's used by a shader, it is important for safety that you make sure that it matches what the shader expects.

However the Vulkan API will likely not provide any way to introspect a shader. Therefore a small SPIR-V analyzer will need to be included in the library that parses the input data.

This situation could be improved after Rust plugins are made stable, so that the analysis is done at compile-time.

## Resources lifetime management

*In this sections, "resources" describes everything that is used by a command buffer: buffers, images, pipelines, dynamic state, etc.*

The rule is that objects must not be written by the CPU nor destroyed while they are still used by the GPU. The library doesn't automatically handle this for you.

This situation can be compared to how Rust handles multithreading. There are two ways:

 - Handle memory "automatically" with an `Arc`, as with `thread::spawn`. The variable can be accessed by both threads, but `Arc` doesn't allow its content to be mutably borrowed.
 - Use regular overhead-free Rust lifetimes, as with `thread::scoped`. The mutable borrow gives exclusive access to the thread that uses the value.

### Stayin' alive

However something else has to be taken into consideration: command buffers also need to somehow make sure that the resources they use are still valid as long as they exist. This could be handled with regular Rust lifetimes, but since Rust doesn't allow structs to borrow themselves my personal opinion is that a `CommandBuffer<'a>` struct would be too cumbersome. You would end up with something like this:

```rust
pub struct GameResources {
    ...
}

pub struct GameCommandBuffers<'a> {
    command_buffer1: vulkano::CommandBuffer<'a>,
    ...
}

fn main() {
    let resources = GameResources::new();
    let command_buffers = GameCommandBuffers::new(&resources);
}
```

If, say, a 3D engine wants to use this library, this split between resources and command buffers would need to be propagated throughout the whole 3D engine. It is just too annoying to deal with. Instead resources should all have their lifetimes managed automatically with a reference counter.

### When to destroy?

Reference counting is similar to using an `Arc`. In Rust, if you use `thread::spawn`, pass an `Arc`, and destroy the original `Arc`, then it's the other thread that becomes responsible for destroying the object. With Vulkan we have a problem: the GPU can't do that.

Therefore we need the reference counter to stay to `1` as long as the GPU still uses our object. I can see three solutions:

 - We handle this internally by storing a list of fences and associated objects, and just a garbage collector we check from time to time whether each object is still in use.
 - Add a `garbage_collect()` method. Similar to the first solution but make it explicit.
 - We give this responsibility to the user by returning a `Fence` object when submitting a command buffer. This `Fence` holds reference counters to the associated resources. When the user destroys the `Fence` it waits for it to be complete and destroys all associated objects. Multiple `Fence` objects can be combined into one.

I prefer the last option.

### Mutability

*Only relevant for buffers and images.*

Since both a command buffer and the user have access to a borrow of the same resource, we have no other choice but to use interior mutability.

Submitting a command buffer should create a fence and add a reference to that fence in each of the resources. When the user attempts to access the resource, it checks the list of fences in order to wait for the resource to stop being used by the GPU. Methods such as `try_write` should be added if the user doesn't want to block.

But should submitting a command buffer wait for fences in the resources they use? The answer would be no if all command buffers are submitted to the same queue, since you would be sure that the first command buffer is over when the second one starts. However we have multiple queues accessible to us (including the DMA and multiple GPUs). Mantle provides semaphore objects for this. It is unknown if Vulkan uses the same mechanism. **This point remains to be seen**.

## Resources state management

With Mantle, all buffers and textures are in a certain state. This is likely going to be the same in Vulkan.
To do certain operations, a resource must be in a given state. For example copying data in RAM to a buffer requires the buffer to be in the `GR_MEMORY_STATE_DATA_TRANSFER` state.

Changing the state of a resource requires using a command in a command buffer, and requires to know what the state of the resource at the time of the execution is. For example to read the same buffer from a shader, you must switch it from the `GR_MEMORY_STATE_DATA_TRANSFER` state to the `GR_MEMORY_STATE_GRAPHICS_SHADER_READ_WRITE` state. This is done by explicitely stating that the buffer currently is in the `GR_MEMORY_STATE_DATA_TRANSFER` state.

The problem is that we need to know what the state of a resource is at the time *when the command buffer is used*, and not at the time when the command buffer is created. Let's say that we build the command buffers A and B. A leaves a given resource in a certain state. B needs to know in which state that resource was left in. Requiring the user to state in which order the command buffers will be executed would be a burden.

Instead to solve this problem, resources will have a "default state".

 - At initialization, a resource is switched to this default state.
 - Command buffers are free to switch the state of a resource, but must restore the resource to its default state at the end.

## Pipelines and dynamic state

There are five type of objects in Mantle that are "dynamic states": the rasterizer, the viewport/scissor, the blender, the depth-stencil, and the multisampling states. In addition to this, there is a pipeline object that holds the list of shaders and more states.

One instance of each of these object types must be binded by command buffers before drawing.

The dynamic state objects are annoying for the programmer to manage. Therefore I think it is a good idea to have the user pass a Rust `enum` that describes the dynamic state, and look for an existing object in a `HashMap`. This hash map would hold `Weak` pointers.

Pipelines, however, would still be created manually.

## Memory views

**To do**

## Context loss

**To do**