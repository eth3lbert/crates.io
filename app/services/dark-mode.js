import Service from '@ember/service';
import { tracked } from '@glimmer/tracking';
import * as localStorage from '../utils/local-storage';

const LIGHT_OR_DARK = new Set(['dark', 'light']);

export default class DarkModeService extends Service {
  @tracked _preferred = localStorage.getItem('dark-mode') ?? 'system';
  @tracked _mode = null;

  setupOnce() {
    let mode = this.preferred;
    const darkQuery = this._darkMQList();
    console.log('[setup]', mode, darkQuery.matches)
    if (mode === 'system') {
      mode = this.detectColorScheme(darkQuery);
    }
    console.log('[setup]', mode)
    this.mode = mode;
    darkQuery.addEventListener('change', e => {
      this.mode = this.detectColorScheme(e);
    });
  }

  get preferred() {
    return this._formatDarkMode(this._preferred, () => 'system');
  }

  set preferred(mode = '') {
    const preferred = this._formatDarkMode(mode, () => 'system');
    this._preferred = preferred;
    if (preferred === 'system') {
      localStorage.removeItem('dark-mode');
    } else {
      localStorage.setItem('dark-mode', preferred);
    }
    this.mode = preferred;
  }

  get mode() {
    return this._formatDarkMode(this._mode, this.detectColorScheme);
  }

  set mode(mode) {
    const darkMode = this._formatDarkMode(mode, this.detectColorScheme);
    this._mode = darkMode;
    if (LIGHT_OR_DARK.has(this.preferred)) {
      document.documentElement.dataset.darkMode = darkMode;
    } else {
      delete document.documentElement.dataset.darkMode;
    }
  }

  _formatDarkMode(mode, defaultCb) {
    return LIGHT_OR_DARK.has(mode) ? mode : defaultCb?.();
  }

  _darkMQList() {
    return window.matchMedia('(prefers-color-scheme: dark)');
  }

  detectColorScheme(darkMQList) {
    return (darkMQList ?? this._darkMQList()).matches ? 'dark' : 'light';
  }
}
