import { CHIP8 } from "chip_8_wasm";
import { memory } from "chip_8_wasm/chip_8_wasm_bg";

const chip8 = CHIP8.new();
const width = 64;
const height = 32;

const PIXEL_SIZE = 5; // px:w
const SET_COLOR = "white";
const UNSET_COLOR = "black";

const canvas = document.getElementById("game_display");

canvas.height = (PIXEL_SIZE + 1) * height + 1;
canvas.width = (PIXEL_SIZE + 1) * width + 1;
const ctx = canvas.getContext("2d");

const renderLoop = () => {
  // Uncomment to debug on each tick
  // debugger;

  chip8.tick();
  drawPixels();

  animation_id = requestAnimationFrame(renderLoop);
};

const getIndex = (row, column) => row * width + column;

const drawPixels = () => {
  const videoPtr = chip8.get_video();
  const pixels = new Uint32Array(memory.buffer, videoPtr, width * height);

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
      if (pixels[idx] == 1) {
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

const fps = new (class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }

  render() {
    // Convert the delta time since the last frame render into a measure
    // of frames per second.
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    this.lastFrameTimeStamp = now;
    const fps = (1 / delta) * 1000;

    // Save only the latest 100 timings.
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }

    // Find the max, min, and mean of our 100 latest timings.
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;

    // Render the statistics.
    this.fps.textContent = `
Frames per Second:
         latest = ${Math.round(fps)}
avg of last 100 = ${Math.round(mean)}
min of last 100 = ${Math.round(min)}
max of last 100 = ${Math.round(max)}
`.trim();
  }
})();

renderLoop();