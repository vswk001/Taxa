// src/types/settings.ts
export interface LlmProvider {
  id: string;
  name: string;
  provider_type: 'claude' | 'openai' | 'openai_compatible' | 'custom';
  api_url: string;
  model_name: string;
  is_default: boolean;
  enabled: boolean;
}

export interface LlmProviderForm {
  name: string;
  provider_type: 'claude' | 'openai' | 'openai_compatible' | 'custom';
  api_url: string;
  api_key: string;
  model_name: string;
  is_default: boolean;
}
