<template>
  <section>
    <canvas ref="what" class="w-100 h-100" />
  </section>
</template>

<script lang="ts" setup>
import { greet, init, Engine } from '@engine/canvas_test'
import { onMounted, ref } from 'vue';

let what = ref<HTMLCanvasElement>();

onMounted(() => {
  if (!what.value) throw new Error('No canvas yet?')

  what.value.width = what.value.clientWidth
  what.value.height = what.value.clientHeight
  
  let engine = init(what.value)
  console.log(engine)
  engine.start();
})
</script>

<style lang="scss" scoped>
canvas {
  background: cornflowerblue;
}
</style>