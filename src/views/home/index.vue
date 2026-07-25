<script setup lang="ts" name="Home">
import { ref } from 'vue'
import Header from './components/Header.vue';
import Search from './components/Search.vue';
import Groups from './components/Groups.vue';
import Content from './components/Content.vue';
import Footer from './components/Footer.vue';

const searchWord = ref('')
const selectedGroup = ref('all')
const viewMode = ref<'list' | 'grid'>('grid')
</script>

<template>
  <div class="w-full h-full overflow-hidden p-4 flex flex-col gap-y-4">
    <Header />
    <Search v-model="searchWord" />
    <Groups v-model="selectedGroup" v-model:view-mode="viewMode" />
    <Content :search-word="searchWord" :group="selectedGroup" :view-mode="viewMode" />
    <Footer />
  </div>
</template>
