// src/types/settings.ts
export type ProviderType = 'claude' | 'openai' | 'openai_compatible' | 'glm' | 'deepseek' | 'minimax' | 'kimi' | 'custom';

export interface LlmProvider {
  id: string;
  name: string;
  provider_type: ProviderType;
  api_url: string;
  model_name: string;
  is_default: boolean;
  enabled: boolean;
}

export interface LlmProviderForm {
  name: string;
  provider_type: ProviderType;
  api_url: string;
  api_key: string;
  model_name: string;
  is_default: boolean;
}
