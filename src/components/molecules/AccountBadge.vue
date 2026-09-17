<script setup lang="ts">
import { GitAccount, getAccountInitials, getProviderLabel } from '../../types/account';

const props = defineProps<{
  account: GitAccount;
  interactive?: boolean;
}>();
</script>

<template>
  <div class="account-badge" :class="{ interactive }">
    <div class="avatar-circle">
      <img v-if="account.avatar_url" :src="account.avatar_url" alt="Avatar" class="avatar-img" />
      <span v-else>{{ getAccountInitials(account) }}</span>
    </div>

    <div class="account-details">
      <div class="account-name-row">
        <span class="account-username">{{ account.name || account.username }}</span>
        <span class="provider-pill" :class="account.provider.toLowerCase()">
          {{ getProviderLabel(account.provider) }}
        </span>
      </div>
      <span class="account-email">{{ account.email }}</span>
    </div>
  </div>
</template>

<style scoped src="../../styles/molecules/AccountBadge.css"></style>
