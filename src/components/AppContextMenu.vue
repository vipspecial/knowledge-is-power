<script setup lang="ts">
interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  danger?: boolean;
  disabled?: boolean;
  separatorBefore?: boolean;
}

defineProps<{
  x: number;
  y: number;
  items: ContextMenuItem[];
}>();

const emit = defineEmits<{
  select: [id: string];
}>();
</script>

<template>
  <div
    class="app-context-menu"
    :style="{ left: `${x}px`, top: `${y}px` }"
    role="menu"
    aria-label="快捷操作"
    @click.stop
    @contextmenu.prevent
  >
    <template v-for="item in items" :key="item.id">
      <span v-if="item.separatorBefore" class="context-menu-separator"></span>
      <button
        type="button"
        role="menuitem"
        :class="{ danger: item.danger }"
        :disabled="item.disabled"
        @click="emit('select', item.id)"
      >
        <span aria-hidden="true">{{ item.icon ?? '' }}</span>
        {{ item.label }}
      </button>
    </template>
  </div>
</template>

<style scoped>
.app-context-menu {
  position: fixed;
  z-index: 120;
  display: grid;
  width: 184px;
  padding: 5px;
  border: 1px solid #d8d3c9;
  border-radius: 10px;
  background: rgb(255 254 250 / 98%);
  box-shadow: 0 12px 34px rgb(42 37 30 / 18%);
  backdrop-filter: blur(16px);
}

.app-context-menu button {
  display: grid;
  height: 30px;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: center;
  gap: 7px;
  padding: 0 8px;
  border: 0;
  border-radius: 7px;
  color: #4d4942;
  background: transparent;
  cursor: pointer;
  font-size:13px;
  text-align: left;
}

.app-context-menu button:hover,
.app-context-menu button:focus-visible {
  outline: none;
  color: var(--accent-strong);
  background: var(--accent-softest);
}

.app-context-menu button.danger {
  color: #a14e46;
}

.app-context-menu button.danger:hover,
.app-context-menu button.danger:focus-visible {
  color: #8c332c;
  background: #fff0ed;
}

.app-context-menu button:disabled {
  cursor: default;
  opacity: 0.42;
}

.app-context-menu button > span {
  color: var(--accent);
  text-align: center;
}

.context-menu-separator {
  height: 1px;
  margin: 4px 5px;
  background: #e5e1d9;
}
</style>
