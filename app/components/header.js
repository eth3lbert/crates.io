import { action } from '@ember/object';
import { inject as service } from '@ember/service';
import Component from '@glimmer/component';

// Six hours.
const SUDO_SESSION_DURATION_MS = 6 * 60 * 60 * 1000;

export default class Header extends Component {
  /** @type {import("../services/session").default} */
  @service session;

  /** @type {import("../services/dark-mode").default} */
  @service darkMode;

  colorSchemes = [
    { mode: 'light', svg: 'sun' },
    { mode: 'dark', svg: 'moon' },
    { mode: 'system', svg: 'color-mode' },
  ];

  @action
  enableSudo() {
    this.session.setSudo(SUDO_SESSION_DURATION_MS);
  }

  @action
  disableSudo() {
    this.session.setSudo(0);
  }

  get preferredDarkMode() {
    return this.colorSchemes.find(c => c.mode === this.darkMode.preferred) ?? {};
  }

  get currentDarkMode() {
    return this.darkMode.mode;
  }

  @action
  setDarkMode(mode) {
    this.darkMode.preferred = mode;
  }
}
