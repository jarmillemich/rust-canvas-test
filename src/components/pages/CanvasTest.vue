<template>
  <section>
    <canvas ref="canvas" class="w-100 h-100" />
  </section>
</template>

<script lang="ts" setup>
import { Engine, init } from '@engine/canvas_test';
import { onMounted, onUnmounted, ref } from 'vue';

let canvas = ref<HTMLCanvasElement>();

let engine: Engine

onMounted(() => {
  if (!canvas.value) throw new Error('No canvas yet?')

  canvas.value.width = canvas.value.clientWidth
  canvas.value.height = canvas.value.clientHeight
 
  
  engine = init(canvas.value)
  engine.connect_local();
  console.log(engine)
  engine.start_web();
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