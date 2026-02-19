export type InputSource = {
  id: string;
  name: string;
  category: string;
};

export type InstalledApp = {
  name: string;
  bundle_id: string;
  path: string;
};

export type AppRule = {
  bundle_id: string;
  app_name: string;
  preferred_input: string;
  is_ai_generated: boolean;
};

export type AppConfig = {
  version: number;
  global_switch: boolean;
  default_input: "en" | "zh" | "keep";
  general: {
    auto_start: boolean;
    hide_dock_icon: boolean;
  };
  rules: AppRule[];
};

export type LLMConfig = {
  api_key: string;
  model: string;
  base_url: string;
};

type TauriWindow = {
  __TAURI__?: object;
  __TAURI_INTERNALS__?: object;
};

export const API = {
  /**
   * Tauri detection & safe invoke
   */
  _isTauri(): boolean {
    if (typeof window === 'undefined') return false;
    const w = window as unknown as TauriWindow;
    return !!(w.__TAURI__ || w.__TAURI_INTERNALS__);
  },
  _invokeFn: undefined as undefined | (<T>(cmd: string, payload?: Record<string, unknown>) => Promise<T>),
  async _invoke<T>(cmd: string, payload?: Record<string, unknown>): Promise<T> {
    if (!API._isTauri()) {
      throw new Error('Tauri runtime not available in web preview');
    }
    if (!API._invokeFn) {
      const core = await import('@tauri-apps/api/core');
      API._invokeFn = core.invoke;
    }
    return API._invokeFn<T>(cmd, payload);
  },
  /**
   * In-browser preview fallbacks
   */
  _mock: {
    config: {
      version: 1,
      global_switch: true,
      default_input: "keep" as const,
      general: {
        auto_start: true,
        hide_dock_icon: false,
      },
      rules: [
        {
          bundle_id: "com.microsoft.VSCode",
          app_name: "VS Code",
          preferred_input: "com.apple.keylayout.ABC",
          is_ai_generated: true,
        },
        {
          bundle_id: "com.tencent.xinWeChat",
          app_name: "WeChat",
          preferred_input: "com.apple.inputmethod.SCIM.ITABC",
          is_ai_generated: true,
        },
        {
          bundle_id: "com.apple.Safari",
          app_name: "Safari",
          preferred_input: "com.apple.inputmethod.SCIM.ITABC",
          is_ai_generated: true,
        },
        {
          bundle_id: "com.apple.Terminal",
          app_name: "Terminal",
          preferred_input: "com.apple.keylayout.ABC",
          is_ai_generated: true,
        },
      ],
    } as AppConfig,
    inputSources: [
      { id: "com.apple.keylayout.ABC", name: "ABC", category: "keyboard" },
      { id: "com.apple.inputmethod.SCIM.ITABC", name: "Chinese - Simplified (Pinyin)", category: "inputmethod" },
    ] as InputSource[],
    llm: {
      api_key: "",
      model: "gpt-4o-mini",
      base_url: "https://api.openai.com/v1",
    } as LLMConfig,
    rescanning: false,
  },
  /**
   * 检查辅助功能权限
   */
  checkPermissions: async (): Promise<boolean> => {
    if (!API._isTauri()) return true;
    return API._invoke('cmd_check_permissions');
  },

  /**
   * 请求辅助功能授权（会触发 macOS 原生授权弹窗）
   */
  requestPermissions: async (): Promise<boolean> => {
    if (!API._isTauri()) return true;
    return API._invoke('cmd_request_permissions');
  },

  /**
   * 打开 macOS 系统设置 (隐私 > 辅助功能)
   */
  openSystemSettings: async (): Promise<void> => {
    if (!API._isTauri()) return;
    return API._invoke('cmd_open_system_settings');
  },

  /**
   * 获取系统输入法列表
   */
  getSystemInputSources: async (): Promise<InputSource[]> => {
    if (!API._isTauri()) return API._mock.inputSources;
    return API._invoke('cmd_get_system_input_sources');
  },

  /**
   * 扫描并预测应用规则
   */
  scanAndPredict: async (inputSources: InputSource[]): Promise<AppRule[]> => {
    if (!API._isTauri()) {
      // Simple mock using heuristic: dev tools -> English, chat/browser -> Chinese
      const pick = (name: string) => {
        const en = "com.apple.keylayout.ABC";
        const zh = "com.apple.inputmethod.SCIM.ITABC";
        if (/code|terminal|intellij|xcode/i.test(name)) return en;
        if (/wechat|whatsapp|qq|dingtalk/i.test(name)) return zh;
        if (/safari|chrome|browser/i.test(name)) return zh;
        return en;
      };
      return API._mock.config.rules.map((r) => ({
        ...r,
        preferred_input: pick(r.app_name),
        is_ai_generated: true,
      }));
    }
    return API._invoke('cmd_scan_and_predict', { inputSources });
  },

  /**
   * 重新扫描并持久化规则（后台完成，避免页面切换导致结果丢失）
   */
  rescanAndSaveRules: async (): Promise<AppRule[]> => {
    if (!API._isTauri()) {
      API._mock.rescanning = true;
      API._mock.rescanning = false;
      return API._mock.config.rules;
    }
    return API._invoke('cmd_rescan_and_save_rules');
  },

  /**
   * 查询规则重扫是否仍在后台进行
   */
  isRescanning: async (): Promise<boolean> => {
    if (!API._isTauri()) {
      return API._mock.rescanning;
    }
    return API._invoke('cmd_is_rescanning');
  },

  /**
   * 保存配置
   */
  saveConfig: async (config: AppConfig): Promise<void> => {
    if (!API._isTauri()) {
      API._mock.config = config;
      try {
        localStorage.setItem('smartime_config', JSON.stringify(config));
      } catch {}
      return;
    }
    return API._invoke('cmd_save_config', { config });
  },

  /**
   * 仅保存规则，避免覆盖其他配置项
   */
  saveRules: async (rules: AppRule[]): Promise<void> => {
    if (!API._isTauri()) {
      API._mock.config = {
        ...API._mock.config,
        rules,
      };
      try {
        localStorage.setItem('smartime_config', JSON.stringify(API._mock.config));
      } catch {}
      return;
    }
    return API._invoke('cmd_save_rules', { rules });
  },

  /**
   * 获取配置
   */
  getConfig: async (): Promise<AppConfig> => {
    if (!API._isTauri()) {
      try {
        const raw = localStorage.getItem('smartime_config');
        if (raw) return JSON.parse(raw);
      } catch {}
      return API._mock.config;
    }
    return API._invoke('cmd_get_config');
  },

  /**
   * 是否存在持久化配置
   */
  hasConfig: async (): Promise<boolean> => {
    if (!API._isTauri()) {
      try {
        return localStorage.getItem('smartime_config') !== null;
      } catch {
        return false;
      }
    }
    return API._invoke('cmd_has_config');
  },

  /**
   * 获取已安装应用列表
   */
  getInstalledApps: async (): Promise<InstalledApp[]> => {
    if (!API._isTauri()) {
      return [
        { name: "VS Code", bundle_id: "com.microsoft.VSCode", path: "/Applications/Visual Studio Code.app" },
        { name: "WeChat", bundle_id: "com.tencent.xinWeChat", path: "/Applications/WeChat.app" },
        { name: "Safari", bundle_id: "com.apple.Safari", path: "/Applications/Safari.app" },
        { name: "Terminal", bundle_id: "com.apple.Terminal", path: "/Applications/Utilities/Terminal.app" },
      ];
    }
    return API._invoke('cmd_get_installed_apps');
  },

  /**
   * 获取 LLM 配置
   */
  getLLMConfig: async (): Promise<LLMConfig> => {
    if (!API._isTauri()) {
      return API._mock.llm;
    }
    return API._invoke('cmd_get_llm_config');
  },

  /**
   * 保存 LLM 配置
   */
  saveLLMConfig: async (config: LLMConfig): Promise<void> => {
    if (!API._isTauri()) {
      API._mock.llm = config;
      try {
        localStorage.setItem('smartime_llm', JSON.stringify(config));
      } catch {}
      return;
    }
    return API._invoke('cmd_save_llm_config', { config });
  },

  /**
   * 检查 LLM 连接
   */
  checkLLMConnection: async (config: LLMConfig): Promise<boolean> => {
    if (!API._isTauri()) {
      // In preview, always return success if API key present
      return !!config.api_key;
    }
    return API._invoke('cmd_check_llm_connection', { config });
  },
};
