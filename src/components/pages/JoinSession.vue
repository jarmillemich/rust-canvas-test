<template>
  <section v-if="connected">
    <canvas ref="canvas" class="w-100 h-100" />
  </section>
  <section v-else>
    <label for="session_name">Session Name</label>
    <input type="text" id="session_name" v-model="sessionName" autofocus @keypress.enter="start_game" />
    
    <label for="client_name">Client Name</label>
    <input type="text" id="client_name" v-model="clientName" />

    <button @click="start_game" :disabled="!sessionName || !clientName">Join</button>
  </section>
</template>

<script lang="ts" setup>
import { useRtcHelper } from '@/usables/useRtcHelper';
import { ConnectionToHost, Engine, init } from '@engine/canvas_test';
import { nextTick, onUnmounted, ref } from 'vue';

let connected = ref(false)
let { joinSession } = useRtcHelper()

let canvas = ref<HTMLCanvasElement>();

let engine: Engine

let sessionName = ref('')
let clientName = ref('asdf')

async function start_game() {
  if (!sessionName.value) throw new Error('No session name yet?')

  // Establish a connection
  let { connection, channel } = await joinSession(sessionName.value, clientName.value)
  let hostConnection = new ConnectionToHost(channel)

  connected.value = true
  await nextTick()

  if (!canvas.value) throw new Error('No canvas yet?')
  canvas.value.width = canvas.value.clientWidth
  canvas.value.height = canvas.value.clientHeight
   
  engine = init(canvas.value)
  engine.connect_as_client(hostConnection)
  console.log(engine)
  // Engine will start itself once it is ready in this mode
  // engine.start();
}

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