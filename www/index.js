import { CHIP8 } from "chip_8_wasm";
import { memory } from "chip_8_wasm/chip_8_wasm_bg";

const chip8 = CHIP8.new();
const width = 64;
const height = 32;

const PIXEL_SIZE = 10; // px:w
const SET_COLOR = "#00FF00";
const UNSET_COLOR = "#000000";

const canvas = document.getElementById("game_display");
const beep = new Audio("/sounds/beep.mp3")

canvas.height = (PIXEL_SIZE + 1) * height + 1;
canvas.width = (PIXEL_SIZE + 1) * width + 1;
const ctx = canvas.getContext("2d");

const renderLoop = () => {
  // Uncomment to debug on each tick
  // debugger;
  for (let i = 0; i < 3; i++) {

    if (chip8.get_sound_timer() !== 0) {
      beep.play();
    } else {
      beep.pause();
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

/*  Keypad:
 *  1 2 3 4
 *  q w e r
 *  a s d f
 *  z x c v   
 */

window.addEventListener("keydown", e => {
  switch (e.key) {
    case "1": {
      chip8.set_key_down(0x1);
      break;
    };
    case "2": {
     chip8.set_key_down(0x2);
     break; 
    };
    case "3": {
      chip8.set_key_down(0x3);
      break;
    };
    case "4": {
      chip8.set_key_down(0xC);
      break;
    };
    case "a": {
      chip8.set_key_down(0x7);
      break;
    };
    case "s": {
     chip8.set_key_down(0x8);
     break; 
    };
    case "d": {
      chip8.set_key_down(0x9);
      break;
    };
    case "f": {
      chip8.set_key_down(0xE);
      break;
    };
    case "q": {
      chip8.set_key_down(0x4);
      break; 
    };
    case "w": {
      chip8.set_key_down(0x5);
      break; 
    };
    case "e": {
      chip8.set_key_down(0x6);
      break;
    };
    case "r": {
      chip8.set_key_down(0xD);
      break;
    };
    case "z": {
      chip8.set_key_down(0xA);
      break;
    };
    case "x": {
      chip8.set_key_down(0x0);
      break;
    };
    case "c": {
      chip8.set_key_down(0xB);
      break;
    };
    case "v": {
      chip8.set_key_down(0xF);
      break;
    };
  };
});

window.addEventListener("keyup", e => {
  switch (e.key) {
    case "1": {
      chip8.set_key_up(0x1);
      break;
    };
    case "2": {
     chip8.set_key_up(0x2);
     break; 
    };
    case "3": {
      chip8.set_key_up(0x3);
      break;
    };
    case "4": {
      chip8.set_key_up(0xC);
      break;
    };
    case "a": {
      chip8.set_key_up(0x7);
      break;
    };
    case "s": {
     chip8.set_key_up(0x8);
     break; 
    };
    case "d": {
      chip8.set_key_up(0x9);
      break;
    };
    case "f": {
      chip8.set_key_up(0xE);
      break;
    };
    case "q": {
      chip8.set_key_up(0x4);
      break; 
    };
    case "w": {
      chip8.set_key_up(0x5);
      break; 
    };
    case "e": {
      chip8.set_key_up(0x6);
      break;
    };
    case "r": {
      chip8.set_key_up(0xD);
      break;
    };
    case "z": {
      chip8.set_key_up(0xA);
      break;
    };
    case "x": {
      chip8.set_key_up(0x0);
      break;
    };
    case "c": {
      chip8.set_key_up(0xB);
      break;
    };
    case "v": {
      chip8.set_key_up(0xF);
      break;
    };
  };
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
      renderLoop();
    })
} 

loadROM();
