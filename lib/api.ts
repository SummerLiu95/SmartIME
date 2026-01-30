import { invoke } from '@tauri-apps/api/core';

export type InputSource = {
  id: string;
  name: string;
  is_enabled: boolean;
};

export type AppRule = {
  bundle_id: string;
  app_name: string;
  preferred_input: string;
  is_ai_generated: boolean;
};

export type LLMConfig = {
  api_key: string;
  model: string;
  base_url: string;
};

export const API = {
  /**
   * 检查辅助功能权限
   */
  checkPermissions: async (): Promise<boolean> => {
    return invoke('cmd_check_permissions');
  },

  /**
   * 打开 macOS 系统设置 (隐私 > 辅助功能)
   */
  openSystemSettings: async (): Promise<void> => {
    return invoke('cmd_open_system_settings');
  },

  /**
   * 获取系统输入法列表
   */
  getSystemInputSources: async (): Promise<InputSource[]> => {
    return invoke('cmd_get_system_input_sources');
  },

  /**
   * 扫描并预测应用规则
   */
  scanAndPredict: async (): Promise<AppRule[]> => {
    return invoke('cmd_scan_and_predict');
  },

  /**
   * 获取 LLM 配置
   */
  getLLMConfig: async (): Promise<LLMConfig> => {
    return invoke('cmd_get_llm_config');
  },

  /**
   * 保存 LLM 配置
   */
  saveLLMConfig: async (config: LLMConfig): Promise<void> => {
    return invoke('cmd_save_llm_config', { config });
  },

  /**
   * 检查 LLM 连接
   */
  checkLLMConnection: async (config: LLMConfig): Promise<boolean> => {
    return invoke('cmd_check_llm_connection', { config });
  },
};
