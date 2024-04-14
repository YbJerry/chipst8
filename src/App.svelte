<script lang="ts">
  import { onMount } from "svelte";
  import { listen, emit } from '@tauri-apps/api/event'

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

  // /**
  //  * A Chip8 keypad is like below:
  //  * 1 2 3 C
  //  * 4 5 6 D
  //  * 7 8 9 E
  //  * A 0 B F
  // **/
  const keyEnum: Record<string, number> = {
    '1': 0x1,
    '2': 0x2,
    '3': 0x3,
    '4': 0xC,
    'q': 0x4,
    'w': 0x5,
    'e': 0x6,
    'r': 0xD,
    'a': 0x7,
    's': 0x8,
    'd': 0x9,
    'f': 0xE,
    'z': 0xA,
    'x': 0x0,
    'c': 0xB,
    'v': 0xF,
  }

  function transKey(key: string) {
    return keyEnum[key];
  }

  async function handleKeydown(event: KeyboardEvent) {
    //console.log(event);
    const key = transKey(event.key);
    // console.log(key)
    if (key !== undefined) {
      await emit('keyEvent', { key: key, press: true });
    }
  }

  async function handleKeyup(event: KeyboardEvent) {
    //console.log(event);
    const key = transKey(event.key);
    if (key !== undefined) {
      await emit('keyEvent', { key: key, press: false });
    }
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