<template>
  <section>
    <canvas ref="what" class="w-100 h-100" />
  </section>
</template>

<script lang="ts" setup>
import { greet } from '@engine/canvas_test'
import { onMounted } from 'vue';

onMounted(greet);
</script>

<style lang="scss" scoped>
canvas {
  background: cornflowerblue;
}
</style>