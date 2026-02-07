<template>
    <div class="h-full flex flex-col bg-background">
        <div class="flex justify-between items-center p-4 border-b border-border">
            <h1 class="text-2xl font-bold">File Manager (v2)</h1>
            <div class="flex gap-2">
                 <Button variant="outline" as-child>
                    <router-link to="/dashboard/files">Switch to Legacy</router-link>
                </Button>
            </div>
        </div>
        <div class="flex-1 overflow-hidden relative">
            <!-- VueFinder Container -->
             <div class="absolute inset-0">
                <vue-finder 
                    id="nusa-finder" 
                    :request="requestConfig"
                    :theme="theme"
                    class="h-full w-full"
                />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useTheme } from '@/design-system';
import { Button } from '@/components/ui/button';

const { isDark } = useTheme();

const theme = computed(() => isDark.value ? 'dark' : 'light');

// Configure request to point to our backend adapter
// Using relative path implies same origin, so cookies are sent automatically
const requestConfig = '/api/vuefinder'; 

</script>

<style>
/* Override VueFinder specific styles if needed to match theme variables */
/* VueFinder uses its own scoped CSS usually, but we might check dark mode integration */
.vf-explorer {
    --vf-color-bg: hsl(var(--background));
    --vf-color-text: hsl(var(--foreground));
    /* Add more overrides if necessary */
}
</style>
