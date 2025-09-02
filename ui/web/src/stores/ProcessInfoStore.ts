import { ProcessTemplate } from '../types';
import { apiClient } from '../api/client';

export interface ProcessInfoStore {
  processTemplates: ProcessTemplate[];
  processTemplatesLoading: boolean;
  processTemplatesError: string | null;
  selectedProcessTemplate: ProcessTemplate | null;
  showProcessTemplateModal: boolean;
  processTemplateId: string;
  processTemplateVariableValues: Record<string, any>;
  
  loadProcessTemplates(): Promise<void>;
  selectProcessTemplate(template: ProcessTemplate): void;
  closeProcessTemplateModal(): void;
  instantiateProcessTemplate(): Promise<void>;
}

export const createProcessInfoStore = (onTemplateCreated: () => Promise<void>): ProcessInfoStore => ({
  processTemplates: [],
  processTemplatesLoading: false,
  processTemplatesError: null,
  selectedProcessTemplate: null,
  showProcessTemplateModal: false,
  processTemplateId: '',
  processTemplateVariableValues: {},

  async loadProcessTemplates() {
    this.processTemplatesLoading = true;
    this.processTemplatesError = null;

    try {
      this.processTemplates = await apiClient.getTemplates();
    } catch (error) {
      this.processTemplatesError = error instanceof Error ? error.message : 'Failed to load templates';
      console.error('Failed to load process templates:', error);
    } finally {
      this.processTemplatesLoading = false;
    }
  },

  selectProcessTemplate(template: ProcessTemplate) {
    this.selectedProcessTemplate = template;
    this.showProcessTemplateModal = true;
    this.processTemplateId = `${template.template_id}-${Date.now()}`;
    this.processTemplateVariableValues = {};

    // Set default values for variables
    template.variables?.forEach(variable => {
      if (variable.default_value !== undefined) {
        this.processTemplateVariableValues[variable.name] = variable.default_value;
      }
    });
  },

  closeProcessTemplateModal() {
    this.showProcessTemplateModal = false;
    this.selectedProcessTemplate = null;
    this.processTemplateId = '';
    this.processTemplateVariableValues = {};
  },

  async instantiateProcessTemplate() {
    if (!this.selectedProcessTemplate) {
      return;
    }

    try {
      await apiClient.instantiateTemplate(
        this.selectedProcessTemplate.template_id,
        this.processTemplateId,
        this.processTemplateVariableValues
      );
      
      // Reload processes after template instantiation
      await onTemplateCreated();
      this.closeProcessTemplateModal();
    } catch (error) {
      this.processTemplatesError = error instanceof Error ? error.message : 'Failed to instantiate template';
      console.error('Failed to instantiate process template:', error);
    }
  }
});