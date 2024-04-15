<script lang="ts">
  import { onMount } from "svelte";
  import { listen, emit } from '@tauri-apps/api/event'

  const audioCtx = new (window.AudioContext || window.webkitAudioContext)();

  // create Oscillator node
  const oscillator = audioCtx.createOscillator();
  const gainNode = audioCtx.createGain();

  oscillator.type = "square";
  oscillator.frequency.value = 440; // value in hertz
  oscillator.connect(gainNode);
  gainNode.connect(audioCtx.destination);

  gainNode.gain.value = 0.02;

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
    if (event.repeat) {
      return;
    }

    if (event.key === '['){
      await emit('speed', {speed: -1});
      return;
    } else if (event.key === ']'){
      await emit('speed', {speed: 1});
      return;
    }

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

    audioCtx.suspend();
    oscillator.start();

    await listen('draw', (event: {
      payload: {
        screen: Array<Array<boolean>>
      }
    }) => {
      console.log(event);
      draw(event.payload.screen as Array<Array<boolean>>);
    });

    await listen('beep', (event: {
      payload: {
        beep: boolean
      }
    }) => {
      console.log(event);
      if (event.payload.beep) {
        console.log('beep');
        audioCtx.resume();
      } else {
        console.log('stop');
        audioCtx.suspend();
      }
    });
  });
</script>

<svelte:window on:keydown={handleKeydown} on:keyup={handleKeyup} />
<main>
  <!-- <div>test</div> -->
  <!-- <audio src="beep.mp3"/> -->
  <!-- <button on:click={() => emit('start')}>Start</button>
  <button on:click={() => emit('stop')}>Stop</button> -->
  <canvas width="640px" height="320px" id="screen"></canvas>
</main>