<template>
    <div class="templates-view">
        <div class="container-xl">
            <div class="page-header">
                <div class="header-content">
                    <h1 class="page-title">{{ t("templates.title") }}</h1>
                </div>
                <button @click="showCreateModal = true" class="btn btn-primary">
                    <IconPlus :size="20" :stroke-width="2" />
                    {{ t("templates.createTemplate") }}
                </button>
            </div>

            <!-- Loading State -->
            <div v-if="templateStore.loading" class="loading-state">
                <div class="spinner-border" role="status"></div>
            </div>

            <!-- Error State -->
            <div v-else-if="templateStore.error" class="error-state">
                <IconAlertTriangle :size="48" :stroke-width="1.5" />
                <h3>{{ t('errors.occurred') }}</h3>
                <p>{{ templateStore.error }}</p>
            </div>

            <!-- Empty State -->
            <div
                v-else-if="templateStore.templateCount === 0"
                class="empty-state"
            >
                <IconTemplate :size="48" :stroke-width="1.5" />
                <h3>{{ t("templates.noTemplates") }}</h3>
                <p>{{ t("templates.noTemplatesDescription") }}</p>
                <button @click="showCreateModal = true" class="btn btn-primary">
                    <IconPlus :size="20" :stroke-width="2" />
                    {{ t("templates.createFirstTemplate") }}
                </button>
            </div>

            <!-- Template Grid -->
            <div v-else class="template-grid">
                <div
                    v-for="template in templateStore.templates"
                    :key="template.template_id"
                    class="template-card"
                >
                    <div class="template-header">
                        <div class="template-icon">
                            <IconTemplate :size="24" :stroke-width="2" />
                        </div>
                        <button
                            @click="deleteTemplate(template)"
                            class="delete-btn"
                            :title="t('common.delete')"
                        >
                            <IconTrash :size="18" :stroke-width="2" />
                        </button>
                    </div>

                    <div class="template-body">
                        <h3 class="template-name">{{ template.name }}</h3>
                        <p
                            v-if="template.description"
                            class="template-description"
                        >
                            {{ template.description }}
                        </p>

                        <div v-if="template.category" class="template-meta">
                            <span class="meta-item">
                                <IconFolder :size="16" :stroke-width="2" />
                                {{ template.category }}
                            </span>
                        </div>

                        <div class="template-command">
                            <code>{{ template.command }}</code>
                            <span v-if="template.args" class="args">{{
                                template.args
                            }}</span>
                        </div>

                        <div
                            v-if="template.tags && template.tags.length > 0"
                            class="template-tags"
                        >
                            <span
                                v-for="tag in template.tags"
                                :key="tag"
                                class="tag"
                            >
                                {{ tag }}
                            </span>
                        </div>
                    </div>

                    <div class="template-footer">
                        <button
                            @click="useTemplate(template)"
                            class="btn btn-primary w-100"
                        >
                            <IconPlayerPlay :size="18" :stroke-width="2" />
                            {{ t("templates.useTemplate") }}
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Create Template Modal -->
        <CreateTemplateModal
            v-model="showCreateModal"
            @created="onTemplateCreated"
        />

        <!-- Use Template Modal -->
        <UseTemplateModal
            v-model="showUseModal"
            :template="selectedTemplate"
            @created="onProcessCreated"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import type { ProcessTemplate, ProcessInfo } from "@/types";
import { useTemplateStore } from "@/stores/template";
import { useToast } from "@/composables/useToast";
import {
    IconTemplate,
    IconPlus,
    IconTrash,
    IconPlayerPlay,
    IconAlertTriangle,
    IconFolder,
} from "@tabler/icons-vue";
import CreateTemplateModal from "@/components/CreateTemplateModal.vue";
import UseTemplateModal from "@/components/UseTemplateModal.vue";
import apiClient from "@/api/client";

const { t } = useI18n();
const router = useRouter();
const { showSuccess, showError } = useToast();
const templateStore = useTemplateStore();

const showCreateModal = ref(false);
const showUseModal = ref(false);
const selectedTemplate = ref<ProcessTemplate | null>(null);

onMounted(async () => {
    await templateStore.loadTemplates();
});

function useTemplate(template: ProcessTemplate) {
    selectedTemplate.value = template;
    showUseModal.value = true;
}

async function deleteTemplate(template: ProcessTemplate) {
    if (!confirm(t("templates.confirmDelete", { name: template.name }))) {
        return;
    }

    try {
        await apiClient.deleteTemplate(template.template_id);
        showSuccess(t("templates.deleteSuccess"));
        await templateStore.loadTemplates();
    } catch (error: any) {
        showError(t("templates.deleteError", { error: error.message }));
    }
}

async function onTemplateCreated() {
    await templateStore.loadTemplates();
}

function onProcessCreated(process: ProcessInfo) {
    showSuccess(t("templates.processCreated", { id: process.id }));
    router.push("/processes");
}
</script>

<style scoped lang="scss">
.templates-view {
    padding: 2rem 0;
}

.page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2rem;
    gap: 1rem;
}

.header-content {
    flex: 1;
    min-width: 0;
}

.page-title {
    font-size: 1.75rem;
    font-weight: 700;
    color: oklch(0.2 0 0);
    margin: 0;

    @media (prefers-color-scheme: dark) {
        color: oklch(0.95 0 0);
    }
}

.btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    font-weight: 500;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;

    &.btn-primary {
        background: var(--vantage-btn-primary-bg);
        color: var(--vantage-btn-primary-fg);

        &:hover {
            background: var(--vantage-btn-primary-hover-bg);
        }
    }

    &.w-100 {
        width: 100%;
        justify-content: center;
    }
}

// Loading State
.loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    color: oklch(0.5 0 0);

    @media (prefers-color-scheme: dark) {
        color: oklch(0.6 0 0);
    }
}

// Error State
.error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    text-align: center;
    color: oklch(0.55 0.18 30);

    @media (prefers-color-scheme: dark) {
        color: oklch(0.75 0.18 30);
    }

    h3 {
        font-size: 1.25rem;
        font-weight: 600;
        margin: 1rem 0 0.5rem;
    }

    p {
        color: oklch(0.5 0 0);
        margin: 0;

        @media (prefers-color-scheme: dark) {
            color: oklch(0.6 0 0);
        }
    }
}

// Empty State
.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    text-align: center;
    color: oklch(0.5 0 0);

    @media (prefers-color-scheme: dark) {
        color: oklch(0.6 0 0);
    }

    h3 {
        font-size: 1.25rem;
        font-weight: 600;
        color: oklch(0.3 0 0);
        margin: 1rem 0 0.5rem;

        @media (prefers-color-scheme: dark) {
            color: oklch(0.8 0 0);
        }
    }

    p {
        margin: 0 0 1.5rem;
    }
}

// Template Grid
.template-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1.5rem;
}

.template-card {
    display: flex;
    flex-direction: column;
    background: oklch(1 0 0);
    border: 1px solid oklch(0.92 0 0);
    border-radius: 8px;
    transition: all 0.15s ease;
    overflow: hidden;

    @media (prefers-color-scheme: dark) {
        background: oklch(0.2 0 0);
        border-color: oklch(0.3 0 0);
    }

    &:hover {
        border-color: oklch(0.85 0 0);
        box-shadow: 0 4px 12px oklch(0 0 0 / 0.08);

        @media (prefers-color-scheme: dark) {
            border-color: oklch(0.35 0 0);
            box-shadow: 0 4px 12px oklch(0 0 0 / 0.3);
        }
    }
}

.template-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1rem 0;
}

.template-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 48px;
    border-radius: 8px;
    background: oklch(0.95 0.02 260 / 0.4);
    color: var(--vantage-btn-primary-bg);

    @media (prefers-color-scheme: dark) {
        background: oklch(0.25 0.04 260 / 0.3);
    }
}

.delete-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: oklch(0.5 0 0);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;

    @media (prefers-color-scheme: dark) {
        color: oklch(0.6 0 0);
    }

    &:hover {
        background: oklch(0.95 0.02 30 / 0.4);
        color: oklch(0.55 0.18 30);

        @media (prefers-color-scheme: dark) {
            background: oklch(0.25 0.04 30 / 0.3);
            color: oklch(0.75 0.18 30);
        }
    }
}

.template-body {
    flex: 1;
    padding: 1rem;
}

.template-name {
    font-size: 1.125rem;
    font-weight: 600;
    color: oklch(0.2 0 0);
    margin: 0 0 0.5rem;

    @media (prefers-color-scheme: dark) {
        color: oklch(0.95 0 0);
    }
}

.template-description {
    font-size: 0.875rem;
    color: oklch(0.5 0 0);
    margin: 0 0 0.75rem;
    line-height: 1.5;

    @media (prefers-color-scheme: dark) {
        color: oklch(0.6 0 0);
    }
}

.template-meta {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
}

.meta-item {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.8125rem;
    color: oklch(0.5 0 0);

    @media (prefers-color-scheme: dark) {
        color: oklch(0.6 0 0);
    }
}

.template-command {
    padding: 0.75rem;
    background: oklch(0.97 0 0);
    border-radius: 6px;
    font-family: "SF Mono", Monaco, Consolas, monospace;
    font-size: 0.8125rem;
    margin-bottom: 0.75rem;
    overflow-x: auto;

    @media (prefers-color-scheme: dark) {
        background: oklch(0.18 0 0);
    }

    code {
        color: oklch(0.3 0 0);

        @media (prefers-color-scheme: dark) {
            color: oklch(0.85 0 0);
        }
    }

    .args {
        color: oklch(0.5 0 0);
        margin-left: 0.5rem;

        @media (prefers-color-scheme: dark) {
            color: oklch(0.6 0 0);
        }
    }
}

.template-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
}

.tag {
    display: inline-flex;
    align-items: center;
    padding: 0.25rem 0.625rem;
    font-size: 0.75rem;
    font-weight: 500;
    background: oklch(0.95 0.02 260 / 0.3);
    color: oklch(0.4 0.04 260);
    border-radius: 4px;

    @media (prefers-color-scheme: dark) {
        background: oklch(0.25 0.04 260 / 0.3);
        color: oklch(0.75 0.04 260);
    }
}

.template-footer {
    padding: 1rem;
    border-top: 1px solid oklch(0.92 0 0);

    @media (prefers-color-scheme: dark) {
        border-top-color: oklch(0.3 0 0);
    }
}

.spinner-border {
    width: 3rem;
    height: 3rem;
    border: 0.25rem solid currentColor;
    border-right-color: transparent;
    border-radius: 50%;
    animation: spinner-border 0.75s linear infinite;
}

@keyframes spinner-border {
    to {
        transform: rotate(360deg);
    }
}
</style>
