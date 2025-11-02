<template>
    <header class="vantage-header">
        <div class="container-xl">
            <div class="header-content">
                <!-- Brand -->
                <router-link to="/" class="brand">
                    <div class="brand-logo">
                        <svg
                            viewBox="0 0 32 32"
                            fill="none"
                            xmlns="http://www.w3.org/2000/svg"
                        >
                            <path
                                d="M16 4L4 10L16 16L28 10L16 4Z"
                                fill="currentColor"
                                opacity="0.9"
                            />
                            <path
                                d="M4 16L16 22L28 16"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                opacity="0.7"
                            />
                            <path
                                d="M4 22L16 28L28 22"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                opacity="0.5"
                            />
                        </svg>
                    </div>
                    <span class="brand-name">{{ t("header.brandName") }}</span>
                </router-link>

                <!-- Navigation -->
                <nav class="header-nav">
                    <router-link
                        v-for="item in navigationItems"
                        :key="item.name"
                        :to="{ name: item.route }"
                        class="nav-link"
                        :class="{ active: route.name === item.route }"
                    >
                        <component
                            :is="item.icon"
                            :size="16"
                            :stroke-width="2"
                        />
                        <span>{{ item.label }}</span>
                    </router-link>
                </nav>

                <!-- Settings Dropdown -->
                <SettingsDropdown />
            </div>
        </div>
    </header>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import {
    IconDashboard,
    IconActivity,
    IconTemplate,
    IconClipboard,
    IconSettings,
} from "@tabler/icons-vue";
import SettingsDropdown from "./SettingsDropdown.vue";

const route = useRoute();
const { t } = useI18n();

const navigationItems = computed(() => [
    {
        name: "dashboard",
        route: "dashboard",
        label: t("navigation.dashboard"),
        icon: IconDashboard,
    },
    {
        name: "processes",
        route: "processes",
        label: t("navigation.processes"),
        icon: IconActivity,
    },
    {
        name: "templates",
        route: "templates",
        label: t("navigation.templates"),
        icon: IconTemplate,
    },
    {
        name: "clipboard",
        route: "clipboard",
        label: t("navigation.clipboard"),
        icon: IconClipboard,
    },
    {
        name: "settings",
        route: "settings",
        label: t("navigation.settings"),
        icon: IconSettings,
    },
]);
</script>

<style scoped lang="scss">
.vantage-header {
    position: sticky;
    top: 0;
    z-index: 1000;
    background: oklch(1 0 0 / 0.8);
    backdrop-filter: blur(12px) saturate(180%);
    border-bottom: 1px solid oklch(0.92 0 0);

    @media (prefers-color-scheme: dark) {
        background: oklch(0.2 0 0 / 0.85);
        border-bottom-color: oklch(0.3 0 0);
    }
}

.header-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 52px;
    gap: 2rem;
}

.brand {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    text-decoration: none;
    color: inherit;
    transition: opacity 0.2s ease;
    flex-shrink: 0;

    &:hover {
        opacity: 0.8;
    }
}

.brand-logo {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    color: var(--vantage-btn-primary-bg);
    flex-shrink: 0;

    svg {
        width: 100%;
        height: 100%;
    }
}

.brand-name {
    font-size: 1.125rem;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.02em;
    color: oklch(0.2 0 0);

    @media (prefers-color-scheme: dark) {
        color: oklch(0.95 0 0);
    }
}

.header-nav {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    justify-content: center;
}

.nav-link {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    text-decoration: none;
    color: oklch(0.5 0 0);
    font-size: 0.875rem;
    font-weight: 500;
    border-radius: 6px;
    transition: all 0.15s ease;
    white-space: nowrap;

    @media (prefers-color-scheme: dark) {
        color: oklch(0.65 0 0);
    }

    &:hover:not(.active) {
        color: oklch(0.3 0 0);
        background: oklch(0.96 0 0);

        @media (prefers-color-scheme: dark) {
            color: oklch(0.85 0 0);
            background: oklch(0.22 0 0);
        }
    }

    &.active {
        color: var(--vantage-btn-primary-bg);
        background: oklch(0.95 0.02 260 / 0.4);

        @media (prefers-color-scheme: dark) {
            background: oklch(0.25 0.04 260 / 0.3);
        }
    }
}

@media (max-width: 768px) {
    .header-content {
        height: 48px;
        gap: 1rem;
    }

    .brand-logo {
        width: 24px;
        height: 24px;
    }

    .brand-name {
        font-size: 1rem;
    }

    .nav-link {
        padding: 0.5rem 0.75rem;
        font-size: 0.8125rem;

        span {
            display: none;
        }
    }
}
</style>
