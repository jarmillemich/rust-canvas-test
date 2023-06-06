<template>
  <section>
    <canvas ref="canvas" class="w-100 h-100" />
  </section>
</template>

<script lang="ts" setup>
import { useRtcHelper } from '@/usables/useRtcHelper';
import { ConnectionToClient, Engine, init } from '@engine/canvas_test';
import { onMounted, onUnmounted, ref } from 'vue';

let { startHostingSession } = useRtcHelper()

let canvas = ref<HTMLCanvasElement>();

let engine: Engine

startHostingSession(({ connection, channel }) => {
  if (!engine) throw new Error('No engine yet?')

  let client = new ConnectionToClient(connection, channel)
  engine.add_client_as_host(client)
})

onMounted(() => {
  if (!canvas.value) throw new Error('No canvas yet?')

  canvas.value.width = canvas.value.clientWidth
  canvas.value.height = canvas.value.clientHeight
  
  
  engine = init(canvas.value)
  engine.connect_as_host();
  console.log(engine)
  engine.start();
})

onUnmounted(() => {
  // Clean up our resources
  console.log('Cleaning up')
  engine.free()
})
</script>

<style lang="scss" scoped>
canvas {
  background: cornflowerblue;
}
</style>