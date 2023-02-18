<template>
  <section>
    <canvas ref="canvas" class="w-100 h-100" />
  </section>
</template>

<script lang="ts" setup>
import { useListener } from '@/usables/useListener';
import { Engine, EventQueue, init } from '@engine/canvas_test';
import { onMounted, onUnmounted, ref } from 'vue';

let canvas = ref<HTMLCanvasElement>();

let engine: Engine

onMounted(() => {
  if (!canvas.value) throw new Error('No canvas yet?')

  canvas.value.width = canvas.value.clientWidth
  canvas.value.height = canvas.value.clientHeight
 
  
  engine = init(canvas.value)
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