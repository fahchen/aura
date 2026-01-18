// Theme definitions for Aura HUD
// 4 theme styles: Liquid Dark, Liquid Light, Solid Dark, Solid Light

export type ThemeStyle = 'system' | 'liquidDark' | 'liquidLight' | 'solidDark' | 'solidLight';

export interface ThemeColors {
  // Text colors
  textPrimary: string;      // Session names
  textSecondary: string;    // Tool labels
  textMuted: string;        // Placeholders, hints
  textHeader: string;       // Session count

  // Icon colors
  iconState: string;        // State icons
  iconTool: string;         // Tool icons
  iconIndicator: string;    // Indicator icon

  // Backgrounds
  containerBg: string;
  contentBg: string;
  rowBg: string;
  rowHoverBg: string;
  indicatorBg: string;

  // Borders
  border: string;
  borderSubtle: string;
  contentHighlight: string;

  // Glass effects
  glassHighlight: string;
  gloss: string;

  // Shadow
  useShadow: boolean;
  shadowOpacity: number;
}

// Liquid Dark: Very see-through frosted glass on dark backgrounds
// Design: subtle border + strong TOP highlight (light reflection from above)
export const liquidDarkTheme: ThemeColors = {
  // Text: light colors for readability on dark backgrounds
  textPrimary: 'rgba(255, 255, 255, 0.95)',
  textSecondary: 'rgba(255, 255, 255, 0.70)',
  textMuted: 'rgba(255, 255, 255, 0.50)',
  textHeader: 'rgba(255, 255, 255, 0.60)',

  // Icons: light colors
  iconState: 'rgba(255, 255, 255, 0.85)',
  iconTool: 'rgba(255, 255, 255, 0.60)',
  iconIndicator: 'rgba(255, 255, 255, 0.95)',

  // Backgrounds: very transparent for see-through glass
  containerBg: 'rgba(255, 255, 255, 0.06)',
  contentBg: 'rgba(255, 255, 255, 0.04)',
  rowBg: 'rgba(255, 255, 255, 0.02)',
  rowHoverBg: 'rgba(255, 255, 255, 0.08)',
  indicatorBg: 'rgba(255, 255, 255, 0.08)',

  // Borders: VERY SUBTLE - barely visible
  border: 'rgba(255, 255, 255, 0.10)',
  borderSubtle: 'rgba(255, 255, 255, 0.06)',
  // Top highlight: STRONG - simulates light reflection from above
  contentHighlight: 'rgba(255, 255, 255, 0.40)',

  // Glass effects: strong top highlight for light source illusion
  glassHighlight: 'rgba(255, 255, 255, 0.50)',
  gloss: 'rgba(255, 255, 255, 0.30)',

  // No shadow for glass
  useShadow: false,
  shadowOpacity: 0,
};

// Liquid Light: Very see-through frosted glass on light backgrounds
// Design: subtle border + strong TOP highlight (light reflection from above)
export const liquidLightTheme: ThemeColors = {
  // Text: dark colors
  textPrimary: '#1A1A1A',
  textSecondary: '#525252',
  textMuted: '#737373',
  textHeader: '#525252',

  // Icons: dark colors
  iconState: '#404040',
  iconTool: '#737373',
  iconIndicator: '#1A1A1A',

  // Backgrounds: very transparent for see-through glass
  containerBg: 'rgba(0, 0, 0, 0.04)',
  contentBg: 'rgba(0, 0, 0, 0.03)',
  rowBg: 'rgba(0, 0, 0, 0.01)',
  rowHoverBg: 'rgba(0, 0, 0, 0.06)',
  indicatorBg: 'rgba(0, 0, 0, 0.06)',

  // Borders: VERY SUBTLE - barely visible
  border: 'rgba(0, 0, 0, 0.08)',
  borderSubtle: 'rgba(0, 0, 0, 0.05)',
  // Top highlight: STRONG - simulates light reflection from above
  contentHighlight: 'rgba(255, 255, 255, 0.55)',

  // Glass effects: strong top highlight for light source illusion
  glassHighlight: 'rgba(255, 255, 255, 0.70)',
  gloss: 'rgba(255, 255, 255, 0.40)',

  // No shadow for glass
  useShadow: false,
  shadowOpacity: 0,
};

// Solid Dark: VS Code / OLED style with shadows
export const solidDarkTheme: ThemeColors = {
  // Text: light colors
  textPrimary: 'rgba(255, 255, 255, 0.92)',
  textSecondary: 'rgba(255, 255, 255, 0.65)',
  textMuted: 'rgba(255, 255, 255, 0.45)',
  textHeader: 'rgba(255, 255, 255, 0.55)',

  // Icons: light colors
  iconState: 'rgba(255, 255, 255, 0.80)',
  iconTool: 'rgba(255, 255, 255, 0.55)',
  iconIndicator: 'rgba(255, 255, 255, 0.92)',

  // Backgrounds: opaque VS Code style
  containerBg: '#1E1E1E',
  contentBg: '#252526',
  rowBg: '#2D2D2D',
  rowHoverBg: '#383838',
  indicatorBg: '#2D2D2D',

  // Borders: subtle
  border: '#3C3C3C',
  borderSubtle: '#333333',
  contentHighlight: '#3C3C3C',

  // No glass effects
  glassHighlight: 'transparent',
  gloss: 'transparent',

  // Shadow for depth
  useShadow: true,
  shadowOpacity: 0.4,
};

// Solid Light: Clean minimal light with shadows
export const solidLightTheme: ThemeColors = {
  // Text: dark colors
  textPrimary: '#171717',
  textSecondary: '#525252',
  textMuted: '#737373',
  textHeader: '#525252',

  // Icons: dark colors
  iconState: '#404040',
  iconTool: '#737373',
  iconIndicator: '#171717',

  // Backgrounds: opaque light
  containerBg: '#F5F5F5',
  contentBg: '#FFFFFF',
  rowBg: '#FAFAFA',
  rowHoverBg: '#F0F0F0',
  indicatorBg: '#FFFFFF',

  // Borders: subtle
  border: '#E5E5E5',
  borderSubtle: '#EBEBEB',
  contentHighlight: '#E5E5E5',

  // No glass effects
  glassHighlight: 'transparent',
  gloss: 'transparent',

  // Shadow for depth
  useShadow: true,
  shadowOpacity: 0.12,
};

// Legacy aliases for backward compatibility
export const darkTheme = liquidDarkTheme;
export const lightTheme = liquidLightTheme;

export type ThemeMode = ThemeStyle; // Alias for compatibility

export function getTheme(style: ThemeStyle, systemIsDark: boolean): ThemeColors {
  switch (style) {
    case 'system':
      return systemIsDark ? liquidDarkTheme : liquidLightTheme;
    case 'liquidDark':
      return liquidDarkTheme;
    case 'liquidLight':
      return liquidLightTheme;
    case 'solidDark':
      return solidDarkTheme;
    case 'solidLight':
      return solidLightTheme;
    default:
      return systemIsDark ? liquidDarkTheme : liquidLightTheme;
  }
}

// Apply theme as CSS variables
export function applyTheme(theme: ThemeColors) {
  const root = document.documentElement;

  root.style.setProperty('--text-primary', theme.textPrimary);
  root.style.setProperty('--text-secondary', theme.textSecondary);
  root.style.setProperty('--text-muted', theme.textMuted);
  root.style.setProperty('--text-header', theme.textHeader);

  root.style.setProperty('--icon-state', theme.iconState);
  root.style.setProperty('--icon-tool', theme.iconTool);
  root.style.setProperty('--icon-indicator', theme.iconIndicator);

  root.style.setProperty('--container-bg', theme.containerBg);
  root.style.setProperty('--content-bg', theme.contentBg);
  root.style.setProperty('--row-bg', theme.rowBg);
  root.style.setProperty('--row-hover-bg', theme.rowHoverBg);
  root.style.setProperty('--indicator-bg', theme.indicatorBg);

  root.style.setProperty('--border', theme.border);
  root.style.setProperty('--border-subtle', theme.borderSubtle);
  root.style.setProperty('--content-highlight', theme.contentHighlight);

  root.style.setProperty('--glass-highlight', theme.glassHighlight);
  root.style.setProperty('--gloss', theme.gloss);

  root.style.setProperty('--use-shadow', theme.useShadow ? '1' : '0');
  root.style.setProperty('--shadow-opacity', theme.shadowOpacity.toString());
}
