<script setup lang="ts" name="Home">
import Header from './components/Header.vue';
import Search from './components/Search.vue';
import Content from './components/Content.vue';
import Footer from './components/Footer.vue';
</script>

<template>
  <div class="w-full h-full overflow-hidden p-4 flex flex-col gap-y-4">
    <Header />
    <Search />
    <Content />
    <Footer />
  </div>
</template>
