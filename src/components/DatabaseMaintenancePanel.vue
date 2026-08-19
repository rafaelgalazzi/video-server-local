<script setup lang="ts">
defineProps<{
  error: string | null
  isClearing: boolean
  isConfirming: boolean
  notice: string | null
}>()

defineEmits<{
  cancel: []
  clear: []
  request: []
}>()
</script>

<template>
  <details class="database-maintenance">
    <summary><span>Advanced</span> Local data</summary>
    <div class="database-maintenance__body">
      <h2>Clear local database</h2>
      <p>
        Remove the indexed library, track preferences, browser sessions, and trusted devices. Your
        files, node identity, and LAN configuration are not deleted.
      </p>

      <div v-if="isConfirming" class="database-maintenance__confirmation" role="alert">
        <p>This cannot be undone. Connected devices will need to pair again.</p>
        <div>
          <button type="button" :disabled="isClearing" @click="$emit('cancel')">Cancel</button>
          <button
            type="button"
            class="danger-action"
            :disabled="isClearing"
            @click="$emit('clear')"
          >
            {{ isClearing ? 'Clearing…' : 'Clear database' }}
          </button>
        </div>
      </div>
      <button v-else type="button" class="danger-action" @click="$emit('request')">
        Clear local database
      </button>

      <p v-if="notice" class="feedback" role="status">{{ notice }}</p>
      <p v-if="error" class="feedback feedback--error" role="alert">{{ error }}</p>
    </div>
  </details>
</template>
