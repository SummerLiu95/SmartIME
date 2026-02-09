export type WindowSize = { width: number; height: number };

const MAIN_SIZE: WindowSize = { width: 896, height: 600 };
const ONBOARDING_SIZE: WindowSize = { width: 384, height: 500 };
const ONBOARDING_LLM_SIZE: WindowSize = { width: 384, height: 560 };
const ONBOARDING_SCAN_SIZE: WindowSize = { width: 384, height: 560 };
const TRAY_SIZE: WindowSize = { width: 300, height: 400 };

export function getWindowSizing(pathname: string) {
  if (pathname.startsWith('/tray')) {
    return { min: TRAY_SIZE, fixed: TRAY_SIZE };
  }
  if (pathname.startsWith('/onboarding/llm')) {
    return { min: ONBOARDING_LLM_SIZE, fixed: ONBOARDING_LLM_SIZE };
  }
  if (pathname.startsWith('/onboarding/scan')) {
    return { min: ONBOARDING_SCAN_SIZE, fixed: ONBOARDING_SCAN_SIZE };
  }
  if (pathname.startsWith('/onboarding')) {
    return { min: ONBOARDING_SIZE, fixed: ONBOARDING_SIZE };
  }
  return { min: MAIN_SIZE, fixed: null };
}
