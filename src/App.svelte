<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from '@tauri-apps/api/event'

  function draw(pixels: Array<Array<boolean>>) {
    const canvas = document.getElementById('screen') as HTMLCanvasElement;
    const width = 10;
    if (canvas.getContext) {
      const ctx = canvas.getContext("2d")!;

      const drawPixel = (x: number, y: number, point: boolean) => {
        ctx.fillStyle = point ? "white" : "black";
        ctx.fillRect(x * width, y * width, width, width);
      }

      for (let y = 0; y < pixels.length; y++) {
        for (let x = 0; x < pixels[y].length; x++) {
          drawPixel(x, y, pixels[y][x]);
        }
      }
    }
  }

  const keyEnum = {
    '1': 0,
    '2': 1,
    '3': 2,
    '4': 3,
    'q': 4,
    'w': 5,
    'e': 6,
    'r': 7,
    'a': 8,
    's': 9,
    'd': 10,
    'f': 11,
    'z': 12,
    'x': 13,
    'c': 14,
    'v': 15,
  }

  function handleKeydown(event: KeyboardEvent) {
    console.log(event);
    
  }

  function handleKeyup(event: KeyboardEvent) {
    console.log(event);
  }

  onMount(async () => {
    console.log('mounted');
    draw(new Array(32).fill(new Array(64).fill(false)));

    const unlisten = await listen('draw', (event: {
      payload: {
        screen: Array<Array<boolean>>
      }
    }) => {
      console.log(event);
      draw(event.payload.screen as Array<Array<boolean>>);
    });
  });
</script>

<svelte:window on:keydown={handleKeydown} on:keyup={handleKeyup} />
<main>
  <!-- <div>test</div> -->
  <canvas width="640px" height="320px" id="screen"></canvas>
</main>