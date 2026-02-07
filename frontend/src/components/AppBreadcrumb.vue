<script setup lang="ts">
import type { Component } from 'vue'

export type BreadcrumbItem = {
    label: string
    icon?: string | Component
    onClick?: () => void
    current?: boolean
}

const props = defineProps<{
    items: BreadcrumbItem[]
    separator?: string
}>()

const isCurrent = (item: BreadcrumbItem, index: number) => {
    return item.current ?? index === props.items.length - 1
}
</script>

<template>
    <nav class="flex items-center gap-2 min-w-0 text-sm text-muted-foreground">
        <template v-for="(item, idx) in items" :key="idx">
            <span v-if="idx > 0" class="text-muted-foreground/40">{{ separator ?? '/' }}</span>
            <component
                :is="item.onClick ? 'button' : 'span'"
                @click="item.onClick && item.onClick()"
                class="flex items-center gap-1.5 truncate transition-colors"
                :class="[
                    item.onClick && !isCurrent(item, idx) ? 'hover:text-primary cursor-pointer' : '',
                    isCurrent(item, idx) ? 'text-foreground font-medium' : 'text-muted-foreground'
                ]"
            >
                <span v-if="typeof item.icon === 'string'" class="material-symbols-outlined text-base">
                    {{ item.icon }}
                </span>
                <component v-else-if="item.icon" :is="item.icon" class="w-4 h-4" />
                <span class="truncate">{{ item.label }}</span>
            </component>
        </template>
    </nav>
</template>
