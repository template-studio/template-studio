const STORAGE_KEY = 'CLIENT_THEME_COLOR';
const DEFAULT_COLOR = '#22c55e';

function hexToRgb(hex: string) {
  const m = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!m) return null;
  return { r: parseInt(m[1], 16), g: parseInt(m[2], 16), b: parseInt(m[3], 16) };
}

function mixWhite(hex: string, ratio: number) {
  const c = hexToRgb(hex);
  if (!c) return hex;
  const mix = (v: number) => Math.round(v + (255 - v) * ratio);
  return `#${[mix(c.r), mix(c.g), mix(c.b)].map(v => v.toString(16).padStart(2, '0')).join('')}`;
}

function darken(hex: string, ratio: number) {
  const c = hexToRgb(hex);
  if (!c) return hex;
  const mix = (v: number) => Math.round(v * (1 - ratio));
  return `#${[mix(c.r), mix(c.g), mix(c.b)].map(v => v.toString(16).padStart(2, '0')).join('')}`;
}

export function applyClientTheme(hexColor: string) {
  const rgb = hexToRgb(hexColor);
  if (!rgb) return;

  const s = document.documentElement.style;
  s.setProperty('--client-theme-color', hexColor);
  s.setProperty('--client-theme-dark', darken(hexColor, 0.2));
  s.setProperty('--client-theme-rgb', `${rgb.r}, ${rgb.g}, ${rgb.b}`);
  s.setProperty('--client-theme-bg-light', mixWhite(hexColor, 0.92));
  s.setProperty('--client-theme-border-light', mixWhite(hexColor, 0.75));

  localStorage.setItem(STORAGE_KEY, hexColor);
}

export function getClientTheme(): string {
  return localStorage.getItem(STORAGE_KEY) || DEFAULT_COLOR;
}

/* ===== 英雄区预设 ===== */
export interface HeroPreset {
  name: string;
  from: string;
  to: string;
}

export const heroPresets: HeroPreset[] = [
  { name: '深空灰', from: '#0f172a', to: '#1e293b' },
  { name: '海洋蓝', from: '#0c1445', to: '#1a237e' },
  { name: '极光紫', from: '#1a0a2e', to: '#2d1b69' },
  { name: '暖夜棕', from: '#1c1917', to: '#292524' },
  { name: '森林绿', from: '#052e16', to: '#14532d' },
  { name: '酒红', from: '#1a0a0a', to: '#4a1020' },
];

const HERO_KEY = 'CLIENT_HERO_PRESET';

export function applyHeroPreset(preset: HeroPreset) {
  const s = document.documentElement.style;
  s.setProperty('--client-hero-from', preset.from);
  s.setProperty('--client-hero-to', preset.to);
  localStorage.setItem(HERO_KEY, JSON.stringify(preset));
}

export function getHeroPreset(): HeroPreset {
  const raw = localStorage.getItem(HERO_KEY);
  if (raw) {
    try { return JSON.parse(raw); } catch {}
  }
  return heroPresets[0];
}

/* ===== 卡片风格预设 ===== */
export interface CardStyle {
  name: string;
  bg: string;
  border: string;
  shadow: string;
  hoverShadow: string;
  radius: string;
}

export const cardStyles: CardStyle[] = [
  {
    name: '简洁边框',
    bg: '#ffffff',
    border: '1px solid #e2e8f0',
    shadow: 'none',
    hoverShadow: '0 8px 24px rgba(15, 23, 42, 0.1)',
    radius: '12px',
  },
  {
    name: '柔和阴影',
    bg: '#ffffff',
    border: 'none',
    shadow: '0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04)',
    hoverShadow: '0 12px 32px rgba(0,0,0,0.1)',
    radius: '14px',
  },
  {
    name: '毛玻璃',
    bg: 'rgba(255, 255, 255, 0.7)',
    border: '1px solid rgba(255, 255, 255, 0.3)',
    shadow: '0 4px 16px rgba(0,0,0,0.06)',
    hoverShadow: '0 12px 32px rgba(0,0,0,0.1)',
    radius: '16px',
  },
  {
    name: '暗色卡片',
    bg: '#1e293b',
    border: '1px solid #334155',
    shadow: '0 2px 8px rgba(0,0,0,0.2)',
    hoverShadow: '0 12px 32px rgba(0,0,0,0.3)',
    radius: '12px',
  },
];

const CARD_KEY = 'CLIENT_CARD_STYLE';

export function applyCardStyle(style: CardStyle) {
  const s = document.documentElement.style;
  s.setProperty('--client-card-bg', style.bg);
  s.setProperty('--client-card-border', style.border);
  s.setProperty('--client-card-shadow', style.shadow);
  s.setProperty('--client-card-hover-shadow', style.hoverShadow);
  s.setProperty('--client-card-radius', style.radius);
  localStorage.setItem(CARD_KEY, JSON.stringify(style));
}

export function getCardStyle(): CardStyle {
  const raw = localStorage.getItem(CARD_KEY);
  if (raw) {
    try { return JSON.parse(raw); } catch {}
  }
  return cardStyles[0];
}
