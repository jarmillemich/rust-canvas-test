<template>
  <header class="flex-auto">
    <nav class="d-flex">
      <router-link to="/">Local</router-link>
      <router-link to="/host">Host</router-link>
      <router-link to="/join">Join</router-link>
    </nav>
  </header>

  <router-view class="flex-grow-1 flex-shrink-0" />
</template>

<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
</script>

<style lang="scss" scoped>
header nav {
  width: 100%;
  background: #248;
  color: #f84;

  a, a:visited, a:active {
    text-decoration: none;
    color: currentColor;
  }

  &>* {
    display: block;
    padding: 0.75rem 0.75rem;

    &:hover {
      background: #48f;
    }

    &:not(:last-child) {
      border-right: 1px solid white;
    }

    &.router-link-exact-active {
      background: #48f;
    }

    
  }
}
</style>
