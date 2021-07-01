
<div align="center">
<h1><code>Boone8</code></h1>
A Chip8 Emulator created in Rust and compiled to WebAssembly 🦀
</div>

## Example Usage
You can get a Chip8 interface up and running by using the example under the examples folder.

## About the JavaScript API

The API exposed by the WASM module allows any form of JavaScript to import the
memory and methods of the CHIP8 structure created in the rust code!

## API Methods
| Name | Params | Usage |
|---|---|:---|
|```tick```||Execute a cycle in the Chip8's CPU. You'd want to call this in a loop in order to start processing the opcodes from the loaded ROM.|
|```reset```||Resets the Chip8's memory back to it's original state.|
|```get_memory```||Returns a pointer to the storage memory. You'd want to store this in a new ```Uint8Array``` in order to retrieve it from the WASM memory.
|```get_video```||Returns a pointer to the video memory. Same as ```get_memory()```, you'd want to use a ```Uint8Array``` for storage.|
|```get_index```||Returns the 16-bit index register used to store memory locations that are used in operations.|
|```get_pc```||Returns the 16-bit program counter that holds the address to the next instruction that the Chip8 will execute.|
|```get_registers```||Returns a pointer to the 16 8-bit registers used to store memory used in operations|
|```get_stack_ptr```||Returns a pointer to the 16 16-bit stack array, used to keep track of the order of execution of the CPU|
|```get_stack_index```||Get the index that keeps track of where we are in the CPU's stack|
|```get_opcode```||Returns the 16-bit opcode that is being executed|
|```get_delay_timer```||Returns the 8-bit timer used to signal delays when it !== 0|
|```get_sound_timer```||Returns the 8-bit timer used to signal that a sound should be played when it !== 0|
|```set_key_down```|```key: usize```|Signals that the key address specified should be set to true| 
|```set_key_up```|```key: usize```|Signals that the key address specified should be set to false| 
