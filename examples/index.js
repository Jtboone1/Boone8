import { CHIP8 } from "chip_8_wasm";
import { memory } from "chip_8_wasm/chip_8_wasm_bg";
import * as keyhandle from "./keypad";

const chip8 = CHIP8.new();
const width = 64;
const height = 32;

const PIXEL_SIZE = 10; // px:w
const SET_COLOR = "#00FF00";
const UNSET_COLOR = "#000000";

const canvas = document.getElementById("game_display");
const beep = new Audio("/sounds/beep.mp3");

canvas.height = (PIXEL_SIZE + 1) * height + 1;
canvas.width = (PIXEL_SIZE + 1) * width + 1;
const ctx = canvas.getContext("2d");

const renderLoop = () => {
  // Uncomment to debug on each tick
  // debugger;

  for (let i = 0; i < 1; i++) {

    if (chip8.get_sound_timer() !== 0) {
      if (beep.paused) {
        beep.play();
      }
    } else {
      if (!beep.paused) {
        beep.pause();
      }
    }
    chip8.tick();
  }
  drawPixels();
  

  requestAnimationFrame(renderLoop);
};

const getIndex = (row, column) => row * width + column;

const drawPixels = () => {
  const videoPtr = chip8.get_video();
  const pixels = new Uint8Array(memory.buffer, videoPtr, width * height);

  ctx.beginPath();

  // fillStyle is separated into two for loops
  // so that we don't need to call fillStyle on 
  // each pixel change, as it's rather expensive. 
  // It's much better for performance to only set 
  // the color twice per render.
  ctx.fillStyle = SET_COLOR;
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      if (pixels[idx] == 0) {
        continue;
      }

      ctx.fillRect(
        col * PIXEL_SIZE + 1,
        row * PIXEL_SIZE + 1,
        PIXEL_SIZE,
        PIXEL_SIZE
      );
    }
  }

  ctx.fillStyle = UNSET_COLOR;
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      if (pixels[idx] != 0) {
        continue;
      }

      ctx.fillRect(
        col * PIXEL_SIZE + 1,
        row * PIXEL_SIZE + 1,
        PIXEL_SIZE,
        PIXEL_SIZE
      );
    }
  }

  ctx.stroke();
};

// Handle keys
window.addEventListener("keydown", e => {
  keyhandle.keydown(e, chip8);
});

window.addEventListener("keyup", e => {
  keyhandle.keyup(e, chip8);
});

const loadROM = () => {
  const memPtr = chip8.get_memory();
  const cpu_memory = new Uint8Array(memory.buffer, memPtr, 4096);

  fetch('/roms/Cave.ch8')
    .then(i => i.arrayBuffer())
    .then(buffer => {
      const romData = new DataView(buffer, 0, buffer.byteLength)
      for (let i = 0; i < romData.byteLength; i++) {
        cpu_memory[0x200 + i] = romData.getUint8(i);
      }

      // After the ROM is loaded into memory, start rendering.
      // If we do this outside of a callback, weird visual bugs
      // will occur.
      renderLoop();
    })
} 

loadROM();
